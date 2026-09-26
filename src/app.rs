//! Interactive runtime: terminal session, input, fixed-step loop, frame pacing.

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event};

use crate::engine::DT;
use crate::game::{Game, Scene, Signal};
use crate::meta::save;
use crate::render::canvas::Canvas;
use crate::term::input::Input;
use crate::term::kitty::{self, MAX_CELLS, Medium, Presenter};
use crate::term::{self, PaneSize, Session};

/// Logical framebuffer area in pixels (about 256x144, reshaped to the pane).
const TARGET_AREA: f32 = 256.0 * 144.0;
/// Most pixels we transmit per frame after integer upscaling.
const PIXEL_BUDGET: f32 = 600_000.0;
/// Most sim ticks run in one loop pass before we drop time.
const MAX_CATCHUP: u32 = 5;

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
    fn scale(&self, medium: Medium) -> usize {
        if let Some(k) = std::env::var("TREX_SCALE").ok().and_then(|v| v.parse::<usize>().ok()) {
            return k.clamp(1, 8);
        }
        if medium == Medium::Direct {
            return 1;
        }
        let fit = (self.disp_w / self.lw as f32).min(self.disp_h / self.lh as f32);
        let budget = (PIXEL_BUDGET / (self.lw * self.lh) as f32).sqrt();
        fit.min(budget).floor().max(1.0) as usize
    }
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
        _ if medium == Medium::Direct => 30.0,
        _ if game.is_live() => 60.0,
        _ => 30.0,
    };
    Duration::from_secs_f32(1.0 / fps)
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
    session.track_image(presenter.id, presenter.tmux());

    let mut layout = Layout::new(term::pane_size()?);
    let mut canvas = Canvas::new(layout.lw, layout.lh);
    let mut game = Game::new(save, save_path, seed, (layout.lw, layout.lh), true);
    let mut input = Input::default();
    let mut stdout = io::stdout();
    let mut out = Vec::with_capacity(1 << 16);
    kitty::placeholders(&mut out, presenter.id, layout.cols, layout.rows);

    let mut focused = true;
    let mut next_tick = Instant::now();
    let mut last_frame = Instant::now() - Duration::from_secs(1);
    let mut dirty = true;

    'main: while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();
        let mut resized = false;
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(k) => input.key(k, now),
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
        if input.force_quit {
            break;
        }
        if resized {
            layout = Layout::new(term::pane_size()?);
            canvas.resize(layout.lw, layout.lh);
            game.set_view((layout.lw, layout.lh));
            out.extend_from_slice(b"\x1b[2J");
            kitty::placeholders(&mut out, presenter.id, layout.cols, layout.rows);
            dirty = true;
        }

        let mut steps = 0;
        while next_tick <= now && steps < MAX_CATCHUP {
            let c = input.controls(now);
            if game.update(&c) == Signal::Quit {
                break 'main;
            }
            next_tick += Duration::from_secs_f32(DT);
            steps += 1;
        }
        if steps == MAX_CATCHUP && next_tick < now {
            next_tick = now;
        }

        if (steps > 0 || dirty)
            && now.duration_since(last_frame) >= frame_interval(&game, focused, presenter.medium)
        {
            game.render(&mut canvas);
            presenter.present(&mut out, &canvas, layout.scale(presenter.medium), layout.cols, layout.rows)?;
            last_frame = now;
            dirty = false;
        }
        if !out.is_empty() {
            stdout.write_all(&out)?;
            stdout.flush()?;
            out.clear();
        }

        event::poll(next_tick.saturating_duration_since(Instant::now()))?;
    }

    presenter.cleanup(&mut out);
    stdout.write_all(&out)?;
    stdout.flush()?;
    drop(session);
    if std::env::var_os("TREX_DEBUG").is_some() {
        eprintln!("trex: graphics medium {:?}, image id {}", presenter.medium, presenter.id);
    }
    Ok(())
}
