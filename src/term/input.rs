//! Keyboard events to game controls.
//!
//! Most terminals (and tmux) never report key release, only a press followed by
//! OS key repeats after an initial delay. A direction therefore counts as held
//! until a short window after its last event: long enough after the first press
//! to bridge the OS repeat delay, short after a repeat. The delay and interval
//! are learned from the repeat stream. Pressing another key (Space to dash)
//! ends a held direction's repeat stream, so the direction stays held for as
//! long as that key's stream lasts; a second direction does the same for the
//! first, so diagonals hold. If the terminal sends kitty-protocol release
//! events, or the OS confirms the key is physically down (macOS, not over SSH),
//! holds become exact.

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::game::Controls;
use crate::term::keystate;

const UP: usize = 0;
const DOWN: usize = 1;
const LEFT: usize = 2;
const RIGHT: usize = 3;

/// Hold window after a repeat event.
const REPEAT_WINDOW: Duration = Duration::from_millis(150);
/// Extra margin on top of the learned initial repeat delay.
const DELAY_MARGIN: Duration = Duration::from_millis(90);
/// Gaps shorter than this are treated as an OS repeat stream.
const REPEAT_MAX_GAP: Duration = Duration::from_millis(120);
/// Menu navigation edges repeat at most this often while a key is held.
const NAV_REPEAT: Duration = Duration::from_millis(140);

#[derive(Clone, Copy, Default)]
struct Hold {
    last: Option<Instant>,
    repeats: u32,
    first_gap: Duration,
    /// Held when the current cover key went down.
    covered: bool,
    /// When the hold began, so the later of two opposite directions wins.
    since: Option<Instant>,
    /// The OS saw this key down when it was pressed, so it also sees the release.
    physical: Option<KeyCode>,
}

pub struct Input {
    holds: [Hold; 4],
    /// The latest non-direction key, whose repeat stream replaced the held
    /// directions' streams.
    cover: Option<(KeyCode, Hold)>,
    /// Kitty keyboard protocol detected: holds end on release events.
    precise: bool,
    /// Physical key state from the OS, if it has one.
    keys: fn(KeyCode) -> Option<bool>,
    delay: Duration,
    interval: Duration,
    edges: Controls,
    /// Last menu navigation edge per direction.
    nav_at: [Option<Instant>; 4],
    /// Ctrl-C: leave immediately.
    pub force_quit: bool,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            holds: [Hold::default(); 4],
            cover: None,
            precise: false,
            keys: keystate::down,
            delay: Duration::from_millis(450),
            interval: Duration::from_millis(40),
            edges: Controls::default(),
            nav_at: [None; 4],
            force_quit: false,
        }
    }
}

fn direction(code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::Char(c) => match c.to_ascii_lowercase() {
            'w' => Some(UP),
            's' => Some(DOWN),
            'a' => Some(LEFT),
            'd' => Some(RIGHT),
            _ => None,
        },
        KeyCode::Up => Some(UP),
        KeyCode::Down => Some(DOWN),
        KeyCode::Left => Some(LEFT),
        KeyCode::Right => Some(RIGHT),
        _ => None,
    }
}

/// A key counts as held until a window after its last event: the initial
/// repeat delay after the first press, a few repeat intervals after that.
fn live(h: &Hold, now: Instant, delay: Duration, interval: Duration) -> bool {
    let Some(last) = h.last else { return false };
    let window = if h.repeats == 0 { delay + DELAY_MARGIN } else { REPEAT_WINDOW.max(interval * 3) };
    now.duration_since(last) < window
}

fn ema(old: Duration, new: Duration) -> Duration {
    old.mul_f32(0.6) + new.mul_f32(0.4)
}

impl Input {
    pub fn key(&mut self, k: KeyEvent, now: Instant) {
        let dir = direction(k.code);
        match k.kind {
            KeyEventKind::Release => {
                self.precise = true;
                if let Some(d) = dir {
                    self.holds[d] = Hold::default();
                }
                return;
            }
            KeyEventKind::Repeat => {
                self.precise = true;
                if let Some(d) = dir {
                    self.holds[d].last = Some(now);
                }
                return;
            }
            KeyEventKind::Press => {}
        }

        if k.modifiers.contains(KeyModifiers::CONTROL) && matches!(k.code, KeyCode::Char('c' | 'C')) {
            self.force_quit = true;
            return;
        }
        if let Some(d) = dir {
            self.press_cover(k.code, now);
            self.press_direction(d, k.code, now);
            return;
        }
        self.press_cover(k.code, now);
        let e = &mut self.edges;
        match k.code {
            KeyCode::Char(' ') | KeyCode::Enter => {
                e.dash = true;
                e.confirm = true;
            }
            KeyCode::Esc => e.pause = true,
            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                'p' => e.pause = true,
                'q' => e.quit = true,
                'h' => e.home = true,
                'y' => e.yes = true,
                'n' => e.no = true,
                '1'..='9' => e.pick = c.to_digit(10).map(|n| n as u8),
                _ => {}
            },
            _ => {}
        }
    }

    fn press_direction(&mut self, d: usize, code: KeyCode, now: Instant) {
        let fresh = !self.held(d, now);
        // Menus step on every tap, and about 7 times a second while held.
        if fresh || self.nav_at[d].is_none_or(|t| now.duration_since(t) >= NAV_REPEAT) {
            self.nav_at[d] = Some(now);
            let e = &mut self.edges;
            match d {
                UP => e.up = true,
                DOWN => e.down = true,
                LEFT => e.left = true,
                _ => e.right = true,
            }
        }
        if fresh {
            let physical = ((self.keys)(code) == Some(true)).then_some(code);
            self.holds[d] = Hold { last: Some(now), since: Some(now), physical, ..Hold::default() };
            return;
        }
        let h = &mut self.holds[d];
        let gap = h.last.map_or(Duration::ZERO, |t| now.duration_since(t));
        h.repeats += 1;
        h.last = Some(now);
        match h.repeats {
            1 => h.first_gap = gap,
            // The second repeat confirms a real repeat stream, so the first gap
            // was the OS initial delay (not a quick double tap).
            2 if gap < REPEAT_MAX_GAP => {
                self.delay = ema(self.delay, h.first_gap)
                    .clamp(Duration::from_millis(150), Duration::from_millis(900));
                self.interval = ema(self.interval, gap);
            }
            _ if gap < REPEAT_MAX_GAP => self.interval = ema(self.interval, gap),
            _ => {}
        }
    }

    fn press_cover(&mut self, code: KeyCode, now: Instant) {
        if let Some((c, h)) = &mut self.cover
            && *c == code
            && live(h, now, self.delay, self.interval)
        {
            h.repeats += 1;
            h.last = Some(now);
            return;
        }
        let held: [bool; 4] = std::array::from_fn(|d| self.held(d, now));
        for (h, held) in self.holds.iter_mut().zip(held) {
            h.covered = held;
        }
        self.cover = Some((code, Hold { last: Some(now), ..Hold::default() }));
    }

    fn held(&self, d: usize, now: Instant) -> bool {
        let h = &self.holds[d];
        if h.last.is_none() {
            return false;
        }
        if let Some(code) = h.physical {
            return (self.keys)(code) == Some(true);
        }
        self.precise
            || live(h, now, self.delay, self.interval)
            || h.covered && self.cover.is_some_and(|(_, c)| live(&c, now, self.delay, self.interval))
    }

    /// Drop all holds (focus lost: releases may never arrive).
    pub fn release_all(&mut self) {
        self.holds = [Hold::default(); 4];
        self.cover = None;
    }

    /// Controls for the next tick. Button edges are handed out once.
    pub fn controls(&mut self, now: Instant) -> Controls {
        let held: [bool; 4] = std::array::from_fn(|d| self.held(d, now));
        for (h, held) in self.holds.iter_mut().zip(held) {
            // A released physical key must not look held when it is pressed again.
            if h.physical.is_some() && !held {
                *h = Hold::default();
            }
        }
        let holds = &self.holds;
        let axis = |neg: usize, pos: usize| match (held[neg], held[pos]) {
            (true, true) if holds[pos].since > holds[neg].since => 1.0,
            (true, true) => -1.0,
            (n, p) => f32::from(u8::from(p)) - f32::from(u8::from(n)),
        };
        let mut c = std::mem::take(&mut self.edges);
        c.move_x = axis(LEFT, RIGHT);
        c.move_y = axis(UP, DOWN);
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;
    use std::cell::Cell;

    thread_local! {
        /// Keys the fake OS reports as down.
        static DOWN: Cell<[bool; 4]> = const { Cell::new([false; 4]) };
    }

    fn fake_keys(code: KeyCode) -> Option<bool> {
        direction(code).map(|d| DOWN.get()[d])
    }

    /// Input on a terminal without release events, and an OS without key state.
    fn input() -> Input {
        Input { keys: |_| None, ..Input::default() }
    }

    fn tap(i: &mut Input, c: char, t: Instant) {
        i.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE), t);
    }

    fn press(i: &mut Input, t: Instant) {
        tap(i, 'd', t);
    }

    #[test]
    fn repeat_stream_holds_then_releases() {
        let t0 = Instant::now();
        let ms = |n: u64| t0 + Duration::from_millis(n);
        let mut i = input();
        press(&mut i, t0);
        // Bridges the OS initial repeat delay.
        assert_eq!(i.controls(ms(400)).move_x, 1.0);
        let mut t = 450;
        while t <= 1000 {
            press(&mut i, ms(t));
            t += 30;
        }
        assert_eq!(i.controls(ms(1000)).move_x, 1.0);
        // Released shortly after the last repeat, not after the long initial window.
        assert_eq!(i.controls(ms(1100)).move_x, 1.0);
        assert_eq!(i.controls(ms(1150)).move_x, 0.0);
    }

    #[test]
    fn menu_edges_on_quick_taps_and_throttled_repeats() {
        let t0 = Instant::now();
        let ms = |n: u64| t0 + Duration::from_millis(n);
        let mut i = input();
        press(&mut i, t0);
        assert!(i.controls(t0).right);
        // A second tap inside the hold window still steps the menu.
        press(&mut i, ms(200));
        assert!(i.controls(ms(200)).right);
        // A fast repeat stream steps at most once per NAV_REPEAT.
        let steps = (0..10)
            .filter(|k| {
                let t = ms(600 + k * 30);
                press(&mut i, t);
                i.controls(t).right
            })
            .count();
        assert_eq!(steps, 2);
    }

    #[test]
    fn direction_stays_held_while_space_repeats() {
        let t0 = Instant::now();
        let ms = |n: u64| t0 + Duration::from_millis(n);
        let mut i = input();
        press(&mut i, t0);
        for t in (450..=600).step_by(30) {
            press(&mut i, ms(t));
        }
        // Space ends the direction's repeat stream and starts its own.
        tap(&mut i, ' ', ms(620));
        assert!(i.controls(ms(620)).dash);
        let mut t = 1070;
        while t <= 2000 {
            tap(&mut i, ' ', ms(t));
            assert_eq!(i.controls(ms(t)).move_x, 1.0);
            t += 30;
        }
        assert_eq!(i.controls(ms(2100)).move_x, 1.0);
        assert_eq!(i.controls(ms(2200)).move_x, 0.0);
    }

    #[test]
    fn diagonal_holds_while_second_direction_repeats() {
        let t0 = Instant::now();
        let ms = |n: u64| t0 + Duration::from_millis(n);
        let mut i = input();
        tap(&mut i, 'w', t0);
        for t in (450..=600).step_by(30) {
            tap(&mut i, 'w', ms(t));
        }
        // D takes over the repeat stream; W stays held for as long as it lasts.
        let mut t = 620;
        tap(&mut i, 'd', ms(t));
        t = 1070;
        while t <= 2000 {
            tap(&mut i, 'd', ms(t));
            let c = i.controls(ms(t));
            assert_eq!((c.move_x, c.move_y), (1.0, -1.0));
            t += 30;
        }
        let c = i.controls(ms(2200));
        assert_eq!((c.move_x, c.move_y), (0.0, 0.0));
    }

    #[test]
    fn physical_keys_hold_through_a_dash_and_release_exactly() {
        let t0 = Instant::now();
        let ms = |n: u64| t0 + Duration::from_millis(n);
        let mut i = Input { keys: fake_keys, ..Input::default() };
        DOWN.set([false, false, false, true]);
        press(&mut i, t0);
        // A tapped Space ends every repeat stream, but D is still down.
        tap(&mut i, ' ', ms(300));
        assert!(i.controls(ms(300)).dash);
        assert_eq!(i.controls(ms(5000)).move_x, 1.0);
        DOWN.set([false; 4]);
        assert_eq!(i.controls(ms(5001)).move_x, 0.0);
        // A key the OS never saw down (typed over SSH) keeps the repeat heuristic.
        tap(&mut i, 'w', ms(6000));
        assert_eq!(i.controls(ms(6300)).move_y, -1.0);
        assert_eq!(i.controls(ms(7000)).move_y, 0.0);
    }

    #[test]
    fn later_opposite_wins_and_release_events_are_exact() {
        let t0 = Instant::now();
        let mut i = input();
        press(&mut i, t0);
        tap(&mut i, 'a', t0 + Duration::from_millis(1));
        assert_eq!(i.controls(t0).move_x, -1.0);
        let mut rel = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        rel.kind = KeyEventKind::Release;
        i.key(rel, t0);
        // D was never released.
        assert_eq!(i.controls(t0).move_x, 1.0);
        rel.code = KeyCode::Char('d');
        i.key(rel, t0);
        assert_eq!(i.controls(t0).move_x, 0.0);
    }
}
