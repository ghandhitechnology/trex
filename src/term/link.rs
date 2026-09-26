//! Flow control for frames sent inline, as over SSH. The terminal answers
//! every frame, so only a round trip's worth of frames ever waits in the
//! connection; on a slow link the frame rate drops instead of the picture
//! falling behind.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Frame rate inline frames aim for.
pub const FPS: f32 = 30.0;
/// A frame or probe whose answer never came is written off after this long;
/// sooner before any answer, since tmux drops the answers to frames that
/// share a burst with a screenful of placeholder text.
const LOST: Duration = Duration::from_millis(1500);
const LOST_UNKNOWN: Duration = Duration::from_millis(500);
/// Frames written off in a row, with no answer between, before frames go out
/// unpaced.
const GIVE_UP: u32 = 4;
/// Round-trip probes: often until a few samples are in, then rarely, since
/// each one briefly drains the link.
const PROBE_FAST: Duration = Duration::from_millis(500);
const PROBE_SLOW: Duration = Duration::from_secs(5);
const PROBE_SETTLED: usize = 4;
/// Probes keep asking while unpaced, so pacing resumes if answers return.
const PROBE_ABSENT: Duration = Duration::from_secs(2);
const RTT_SAMPLES: usize = 16;
const MAX_IN_FLIGHT: usize = 6;
/// Smoothing for the carry time.
const ALPHA: f32 = 0.3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Replies {
    Unknown,
    Working,
    Absent,
}

pub struct Link {
    replies: Replies,
    /// Frames written off since the last answer.
    lost: u32,
    /// Sent time of each unanswered frame, oldest first.
    in_flight: VecDeque<Instant>,
    probe_sent: Option<Instant>,
    last_probe: Option<Instant>,
    rtts: VecDeque<Duration>,
    last_answer: Option<Instant>,
    /// Smoothed seconds the link spends carrying one frame.
    carry: Option<f32>,
}

impl Link {
    pub fn new() -> Self {
        Link {
            replies: Replies::Unknown,
            lost: 0,
            in_flight: VecDeque::new(),
            probe_sent: None,
            last_probe: None,
            rtts: VecDeque::new(),
            last_answer: None,
            carry: None,
        }
    }

    pub fn wants_replies(&self) -> bool {
        self.replies != Replies::Absent
    }

    fn rtt(&self) -> Option<Duration> {
        self.rtts.iter().min().copied()
    }

    /// Frames allowed in flight: enough to keep the link busy for a round
    /// trip, plus the one being carried. A frame that takes longer than a
    /// frame interval to carry counts for more of the round trip.
    fn window(&self) -> usize {
        let rtt = self.rtt().map_or(0.0, |d| d.as_secs_f32());
        let per_frame = self.carry.unwrap_or(0.0).max(1.0 / FPS);
        ((rtt / per_frame).ceil() as usize + 1).clamp(2, MAX_IN_FLIGHT)
    }

    /// Whether a frame may go out now.
    pub fn ready(&mut self, now: Instant) -> bool {
        let lost = if self.replies == Replies::Unknown { LOST_UNKNOWN } else { LOST };
        while self.in_flight.front().is_some_and(|&t| now - t > lost) {
            self.in_flight.pop_front();
            self.lost += 1;
        }
        if self.probe_sent.is_some_and(|t| now - t > lost) {
            self.probe_sent = None;
        }
        if self.lost >= GIVE_UP && self.replies != Replies::Absent {
            self.replies = Replies::Absent;
            self.in_flight.clear();
        }
        if self.replies == Replies::Absent {
            return true;
        }
        // An overdue probe waits for the link to drain, so it times the path
        // and not the queue.
        if self.probe_overdue(now) {
            return self.in_flight.is_empty();
        }
        self.in_flight.len() < self.window()
    }

    fn probe_overdue(&self, now: Instant) -> bool {
        let every = match self.replies {
            Replies::Absent => PROBE_ABSENT,
            _ if self.rtts.len() < PROBE_SETTLED => PROBE_FAST,
            _ => PROBE_SLOW,
        };
        self.probe_sent.is_none() && self.last_probe.is_none_or(|t| now - t >= every)
    }

    /// Whether to send a round-trip probe ahead of this frame.
    pub fn probe_due(&mut self, now: Instant) -> bool {
        let due = self.probe_overdue(now) && (self.replies == Replies::Absent || self.in_flight.is_empty());
        if due {
            self.probe_sent = Some(now);
            self.last_probe = Some(now);
        }
        due
    }

    pub fn sent(&mut self, now: Instant) {
        if self.wants_replies() {
            self.in_flight.push_back(now);
        }
    }

    fn heard(&mut self) {
        self.replies = Replies::Working;
        self.lost = 0;
    }

    pub fn probe_answered(&mut self, now: Instant) {
        self.heard();
        if let Some(t) = self.probe_sent.take() {
            if self.rtts.len() == RTT_SAMPLES {
                self.rtts.pop_front();
            }
            self.rtts.push_back(now - t);
        }
    }

    pub fn frame_answered(&mut self, now: Instant) {
        self.heard();
        let Some(sent) = self.in_flight.pop_front() else { return };
        // Queued behind the previous frame, its carry is the answer spacing;
        // sent into an idle link, it is the latency past the round trip.
        let carry = match (self.last_answer, self.rtt()) {
            (Some(prev), _) if sent < prev => now - prev,
            (_, Some(rtt)) => (now - sent).saturating_sub(rtt),
            _ => now - sent,
        }
        .as_secs_f32();
        self.last_answer = Some(now);
        self.carry = Some(self.carry.map_or(carry, |c| c + ALPHA * (carry - c)));
    }

    pub fn summary(&self) -> String {
        format!(
            "replies {:?}, rtt {} ms, carry {} ms, window {}",
            self.replies,
            self.rtt().map_or("?".into(), |d| format!("{:.0}", d.as_secs_f32() * 1000.0)),
            self.carry.map_or("?".into(), |c| format!("{:.0}", c * 1000.0)),
            self.window(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays 40 s of 30 fps frames of `kb` over a link of `mbit` and `rtt_ms`.
    /// The first `lose` answers never arrive, as under tmux. Returns the
    /// frames sent and the worst answer latency in the last 10 s.
    fn simulate(kb: f32, mbit: f32, rtt_ms: u64, mut lose: usize) -> (u32, Duration) {
        let t0 = Instant::now();
        let mut link = Link::new();
        let bytes_per_s = mbit * 1e6 / 8.0;
        let rtt = Duration::from_millis(rtt_ms);
        let mut busy_until = t0;
        // (arrival of the answer, is probe, sent at)
        let mut answers: Vec<(Instant, bool, Instant)> = Vec::new();
        let (mut sent, mut worst) = (0, Duration::ZERO);
        let mut next_frame = t0;
        for ms in 0..40_000 {
            let now = t0 + Duration::from_millis(ms);
            answers.sort_by_key(|a| a.0);
            while answers.first().is_some_and(|a| a.0 <= now) {
                let (at, probe, at_send) = answers.remove(0);
                if probe {
                    link.probe_answered(at);
                } else {
                    link.frame_answered(at);
                    if ms > 30_000 {
                        worst = worst.max(at - at_send);
                    }
                }
            }
            if now < next_frame {
                continue;
            }
            next_frame += Duration::from_secs_f32(1.0 / FPS);
            if !link.ready(now) {
                continue;
            }
            let mut send = |bytes: f32, probe: bool| {
                let start = busy_until.max(now);
                busy_until = start + Duration::from_secs_f32(bytes / bytes_per_s);
                if lose > 0 {
                    lose -= 1;
                } else {
                    answers.push((busy_until + rtt, probe, now));
                }
            };
            if link.probe_due(now) {
                send(100.0, true);
            }
            send(kb * 1024.0, false);
            link.sent(now);
            sent += 1;
        }
        (sent, worst)
    }

    #[test]
    fn slow_links_drop_frames_instead_of_falling_behind() {
        // A 1x frame mid-run is about 23 KB.
        for (mbit, rtt) in [(1.0, 80), (3.0, 150), (20.0, 30), (200.0, 10)] {
            let (sent, worst) = simulate(23.0, mbit, rtt, 3);
            let carry = Duration::from_secs_f32(23.0 * 1024.0 * 8.0 / (mbit * 1e6));
            assert!(worst < Duration::from_millis(rtt) + carry * 3, "{mbit} Mbit/s, {rtt} ms: {worst:?}");
            if mbit >= 20.0 {
                assert!(sent > 1100, "{mbit} Mbit/s, {rtt} ms: {sent} frames");
            }
        }
    }

    #[test]
    fn a_terminal_that_never_answers_gets_unpaced_frames() {
        let t0 = Instant::now();
        let mut link = Link::new();
        let mut sent = 0;
        for ms in (0..3000).step_by(33) {
            let now = t0 + Duration::from_millis(ms);
            if link.ready(now) {
                link.probe_due(now);
                link.sent(now);
                sent += 1;
            }
        }
        assert!(!link.wants_replies());
        assert!(sent > 60, "{sent}");
    }
}
