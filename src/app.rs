//! Interactive runtime: terminal session, input, fixed-step loop, frame pacing.

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event};

use crate::engine::DT;
use crate::game::{Game, Scene, Signal};
use crate::meta::save;
use crate::render::canvas::Canvas;
use crate::render::palette::Color;
use crate::term::input::Input;
use crate::term::kitty::{self, AnswerFilter, MAX_CELLS, Medium, Presenter};
use crate::term::{self, PaneSize, Session, link};

/// Logical framebuffer area in pixels (about 256x144, reshaped to the pane).
const TARGET_AREA: f32 = 256.0 * 144.0;
/// Most pixels we transmit per frame after integer upscaling.
pub const PIXEL_BUDGET: f32 = 600_000.0;
/// Most sim ticks run in one loop pass before we drop time.
const MAX_CATCHUP: u32 = 5;
/// Timer jitter tolerated when pacing frames, so a late wake followed by an
/// on-time one does not skip a frame.
const FRAME_SLACK: Duration = Duration::from_millis(2);

/// How the pane maps to the framebuffer.
struct Layout {
    cols: u16,
    rows: u16,
    lw: i32,
    lh: i32,
    disp_w: f32,
    disp_h: f32,
}

impl Layout {
    fn new(p: PaneSize) -> Self {
        let (cols, rows) = (p.cols.clamp(1, MAX_CELLS), p.rows.clamp(1, MAX_CELLS));
        // Unknown pixel size: assume cells twice as tall as wide.
        let (cw, ch) = if p.px_w > 0 && p.px_h > 0 {
            (p.px_w as f32 / p.cols as f32, p.px_h as f32 / p.rows as f32)
        } else {
            (8.0, 16.0)
        };
        let (disp_w, disp_h) = (cols as f32 * cw, rows as f32 * ch);
        let aspect = (disp_w / disp_h).clamp(0.6, 3.2);
        let lw = (TARGET_AREA * aspect).sqrt().round() as i32;
        let lh = (lw as f32 / aspect).round() as i32;
        Layout { cols, rows, lw, lh, disp_w, disp_h }
    }

    /// Integer upscale so the terminal's own (linear) scaling stays small.
    fn scale(&self) -> usize {
        let fit = (self.disp_w / self.lw as f32).min(self.disp_h / self.lh as f32);
        let budget = (PIXEL_BUDGET / (self.lw * self.lh) as f32).sqrt();
        fit.min(budget).floor().max(1.0) as usize
    }
}

/// `TREX_SCALE` pins the upscale.
fn forced_scale() -> Option<usize> {
    std::env::var("TREX_SCALE").ok().and_then(|v| v.parse::<usize>().ok()).map(|k| k.clamp(1, 8))
}

fn pick_medium() -> (Medium, bool) {
    if let Some(m) = std::env::var("TREX_GFX").ok().as_deref().and_then(Medium::parse) {
        return (m, true);
    }
    let remote = std::env::var_os("SSH_CONNECTION").is_some() || std::env::var_os("SSH_TTY").is_some();
    (if remote { Medium::Direct } else { Medium::Shm }, false)
}

fn frame_interval(game: &Game, focused: bool, medium: Medium) -> Duration {
    let fps = match (&game.scene, focused) {
        (_, false) | (Scene::Paused, _) => 8.0,
        _ if medium == Medium::Direct => link::FPS,
        _ if game.is_live() => 60.0,
        _ => 30.0,
    };
    Duration::from_secs_f32(1.0 / fps)
}

/// Paused or unfocused outside a run: nothing moves on its own, so the loop
/// wakes only for frames and input instead of every tick.
fn idle(game: &Game, focused: bool) -> bool {
    !game.is_live() && (!focused || matches!(game.scene, Scene::Paused))
}

pub fn run() -> io::Result<()> {
    let save_path = save::default_path();
    let save = save_path.as_deref().map(save::load).unwrap_or_default();
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);

    let stop = Arc::new(AtomicBool::new(false));
    for sig in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGHUP, signal_hook::consts::SIGINT] {
        signal_hook::flag::register(sig, Arc::clone(&stop))?;
    }

    let (medium, forced) = pick_medium();
    let mut presenter = Presenter::new(medium, forced, term::in_tmux());
    let session = Session::enter()?;
    // Restoring the terminal also deletes this image.
    session.track_image(presenter.id, presenter.tmux());

    let layout = Layout::new(term::pane_size()?);
    let mut game = Game::new(save, save_path, seed, (layout.lw, layout.lh), true);
    let result = play(&mut game, &mut presenter, layout, &stop);
    // Ctrl-C, a signal, or a dead terminal still banks the run in progress.
    game.quit();
    drop(session);
    if std::env::var_os("TREX_DEBUG").is_some() {
        eprintln!("trex: graphics medium {:?}, image id {}", presenter.medium, presenter.id);
        if presenter.medium == Medium::Direct {
            eprintln!("trex: link {}", presenter.link_summary());
        }
    }
    result
}

/// What the input thread hands the loop, stamped on arrival.
enum Incoming {
    Event(Event),
    /// The terminal answered a graphics command for this image id.
    Answer(u32),
}

/// Terminal events, read on their own thread and stamped on arrival.
/// Graphics answers are split out of the key stream here.
///
/// crossterm's reader spins forever once the terminal hangs up (a closed pane
/// reads as endless EOF), so it must never block the game loop. The loop
/// notices the hangup through failed writes or SIGHUP and exits; the stuck
/// thread dies with the process.
fn spawn_input() -> Receiver<(Incoming, Instant)> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut filter = AnswerFilter::default();
        let mut pass = Vec::new();
        while let Ok(ev) = event::read() {
            let at = Instant::now();
            let answer = filter.feed(ev, &mut pass).map(Incoming::Answer);
            let out = pass.drain(..).map(Incoming::Event).chain(answer);
            if out.map(|m| tx.send((m, at))).any(|r| r.is_err()) {
                break;
            }
        }
    });
    rx
}

/// The fixed-step loop, until the player quits, Ctrl-C, or a signal.
fn play(game: &mut Game, presenter: &mut Presenter, mut layout: Layout, stop: &AtomicBool) -> io::Result<()> {
    let mut canvas = Canvas::new(layout.lw, layout.lh);
    let mut input = Input::default();
    let events = spawn_input();
    let mut woke = None;
    let mut stdout = io::stdout();
    let mut out = Vec::with_capacity(1 << 16);
    kitty::placeholders(&mut out, presenter.id, layout.cols, layout.rows);

    let mut focused = true;
    let mut next_tick = Instant::now();
    let mut last_frame = Instant::now() - Duration::from_secs(1);
    let mut dirty = true;
    // Last frame the terminal was sent; an identical frame is skipped.
    let mut shown: Vec<Color> = Vec::new();
    let forced = forced_scale();

    while !stop.load(Ordering::Relaxed) {
        let mut resized = false;
        for (msg, at) in woke.take().into_iter().chain(events.try_iter()) {
            let ev = match msg {
                Incoming::Event(ev) => ev,
                Incoming::Answer(id) => {
                    presenter.answered(id, at);
                    continue;
                }
            };
            match ev {
                Event::Key(k) => input.key(k, at),
                Event::FocusLost => {
                    focused = false;
                    game.focus_lost();
                    input.release_all();
                }
                Event::FocusGained => focused = true,
                Event::Resize(..) => resized = true,
                _ => {}
            }
        }
        let now = Instant::now();
        if input.force_quit {
            return Ok(());
        }
        if resized {
            layout = Layout::new(term::pane_size()?);
            canvas.resize(layout.lw, layout.lh);
            game.set_view((layout.lw, layout.lh));
            out.extend_from_slice(b"\x1b[2J");
            shown.clear();
            kitty::placeholders(&mut out, presenter.id, layout.cols, layout.rows);
            dirty = true;
        }

        let interval = frame_interval(game, focused, presenter.medium);
        // Idle wakes are a frame apart, so catch up a whole frame of ticks.
        let max_steps = MAX_CATCHUP.max((interval.as_secs_f32() / DT).ceil() as u32);
        let mut steps = 0;
        while next_tick <= now && steps < max_steps {
            let c = input.controls(now);
            if game.update(&c) == Signal::Quit {
                return Ok(());
            }
            next_tick += Duration::from_secs_f32(DT);
            steps += 1;
        }
        if steps == max_steps && next_tick < now {
            next_tick = now;
        }

        if (steps > 0 || dirty) && now.duration_since(last_frame) + FRAME_SLACK >= interval {
            game.render(&mut canvas);
            let changed = canvas.px != shown || !presenter.settled();
            let (k, fixed) = forced.map_or((layout.scale(), false), |k| (k, true));
            dirty = false;
            if changed {
                if presenter.present(&mut out, &canvas, k, fixed, layout.cols, layout.rows)? {
                    shown.clone_from(&canvas.px);
                } else {
                    // Dropped while the terminal catches up; try again next frame.
                    dirty = true;
                }
            }
            last_frame = now;
        }
        if !out.is_empty() {
            stdout.write_all(&out)?;
            stdout.flush()?;
            out.clear();
        }

        let wake = if idle(game, focused) { next_tick.max(last_frame + interval) } else { next_tick };
        match events.recv_timeout(wake.saturating_duration_since(Instant::now())) {
            Ok(ev) => woke = Some(ev),
            Err(RecvTimeoutError::Timeout) => {}
            // The reader failed: the terminal is gone.
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
    Ok(())
}
