//! Terminal session: raw mode, alternate screen, focus reports, keyboard
//! enhancement, and a restore path that runs on normal exit, panic, and signals.

pub mod input;
pub mod keystate;
pub mod kitty;
pub mod link;

use std::io::{self, Write};
use std::sync::Once;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crossterm::event::{
    DisableFocusChange, EnableFocusChange, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::style::ResetColor;
use crossterm::terminal::{
    self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};

static ACTIVE: AtomicBool = AtomicBool::new(false);
static IMAGE_ID: AtomicU32 = AtomicU32::new(0);
static IMAGE_TMUX: AtomicBool = AtomicBool::new(false);

/// Owns the terminal while the game runs. Dropping it restores the terminal.
pub struct Session;

impl Session {
    pub fn enter() -> io::Result<Session> {
        install_panic_hook();
        terminal::enable_raw_mode()?;
        ACTIVE.store(true, Ordering::SeqCst);
        let mut out = io::stdout();
        execute!(
            out,
            EnterAlternateScreen,
            cursor::Hide,
            DisableLineWrap,
            EnableFocusChange,
            Clear(ClearType::All),
        )?;
        // Terminals with the kitty keyboard protocol then report key releases.
        execute!(
            out,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )?;
        Ok(Session)
    }

    /// Remember the live image so the restore path can delete it.
    pub fn track_image(&self, id: u32, tmux: bool) {
        IMAGE_TMUX.store(tmux, Ordering::SeqCst);
        IMAGE_ID.store(id, Ordering::SeqCst);
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        restore();
    }
}

/// Idempotent terminal restore. Safe from the panic hook.
pub fn restore() {
    if !ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }
    let mut out = io::stdout();
    let id = IMAGE_ID.swap(0, Ordering::SeqCst);
    if id != 0 {
        let mut buf = Vec::new();
        kitty::delete(&mut buf, id, IMAGE_TMUX.load(Ordering::SeqCst));
        let _ = out.write_all(&buf);
    }
    let _ = execute!(
        out,
        PopKeyboardEnhancementFlags,
        DisableFocusChange,
        ResetColor,
        Clear(ClearType::All),
        EnableLineWrap,
        cursor::Show,
        LeaveAlternateScreen,
    );
    let _ = terminal::disable_raw_mode();
    let _ = out.flush();
}

fn install_panic_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            prev(info);
        }));
    });
}

#[derive(Clone, Copy, Debug)]
pub struct PaneSize {
    pub cols: u16,
    pub rows: u16,
    /// Pixel size of the pane; 0 when the terminal does not report it.
    pub px_w: u16,
    pub px_h: u16,
}

pub fn pane_size() -> io::Result<PaneSize> {
    match terminal::window_size() {
        Ok(ws) if ws.columns > 0 && ws.rows > 0 => {
            Ok(PaneSize { cols: ws.columns, rows: ws.rows, px_w: ws.width, px_h: ws.height })
        }
        _ => {
            let (cols, rows) = terminal::size()?;
            Ok(PaneSize { cols, rows, px_w: 0, px_h: 0 })
        }
    }
}

pub fn in_tmux() -> bool {
    std::env::var_os("TMUX").is_some_and(|v| !v.is_empty())
}
