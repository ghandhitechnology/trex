//! Kitty graphics output: one image id, re-transmitted every frame and shown
//! through Unicode placeholders so it stays attached to the pane (works in tmux).
//!
//! Frames go through shared memory or a temp file when the terminal shares our
//! machine, so only a tiny escape crosses tmux. If the terminal never consumes
//! those, we fall back to zlib + base64 chunks inside the escapes.

use std::ffi::CString;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::render::canvas::Canvas;
use crate::term::link::Link;

/// Row/column diacritics from kitty's rowcolumn-diacritics.txt; index = number.
pub const DIACRITICS: [char; 297] = [
    '\u{305}',
    '\u{30d}',
    '\u{30e}',
    '\u{310}',
    '\u{312}',
    '\u{33d}',
    '\u{33e}',
    '\u{33f}',
    '\u{346}',
    '\u{34a}',
    '\u{34b}',
    '\u{34c}',
    '\u{350}',
    '\u{351}',
    '\u{352}',
    '\u{357}',
    '\u{35b}',
    '\u{363}',
    '\u{364}',
    '\u{365}',
    '\u{366}',
    '\u{367}',
    '\u{368}',
    '\u{369}',
    '\u{36a}',
    '\u{36b}',
    '\u{36c}',
    '\u{36d}',
    '\u{36e}',
    '\u{36f}',
    '\u{483}',
    '\u{484}',
    '\u{485}',
    '\u{486}',
    '\u{487}',
    '\u{592}',
    '\u{593}',
    '\u{594}',
    '\u{595}',
    '\u{597}',
    '\u{598}',
    '\u{599}',
    '\u{59c}',
    '\u{59d}',
    '\u{59e}',
    '\u{59f}',
    '\u{5a0}',
    '\u{5a1}',
    '\u{5a8}',
    '\u{5a9}',
    '\u{5ab}',
    '\u{5ac}',
    '\u{5af}',
    '\u{5c4}',
    '\u{610}',
    '\u{611}',
    '\u{612}',
    '\u{613}',
    '\u{614}',
    '\u{615}',
    '\u{616}',
    '\u{617}',
    '\u{657}',
    '\u{658}',
    '\u{659}',
    '\u{65a}',
    '\u{65b}',
    '\u{65d}',
    '\u{65e}',
    '\u{6d6}',
    '\u{6d7}',
    '\u{6d8}',
    '\u{6d9}',
    '\u{6da}',
    '\u{6db}',
    '\u{6dc}',
    '\u{6df}',
    '\u{6e0}',
    '\u{6e1}',
    '\u{6e2}',
    '\u{6e4}',
    '\u{6e7}',
    '\u{6e8}',
    '\u{6eb}',
    '\u{6ec}',
    '\u{730}',
    '\u{732}',
    '\u{733}',
    '\u{735}',
    '\u{736}',
    '\u{73a}',
    '\u{73d}',
    '\u{73f}',
    '\u{740}',
    '\u{741}',
    '\u{743}',
    '\u{745}',
    '\u{747}',
    '\u{749}',
    '\u{74a}',
    '\u{7eb}',
    '\u{7ec}',
    '\u{7ed}',
    '\u{7ee}',
    '\u{7ef}',
    '\u{7f0}',
    '\u{7f1}',
    '\u{7f3}',
    '\u{816}',
    '\u{817}',
    '\u{818}',
    '\u{819}',
    '\u{81b}',
    '\u{81c}',
    '\u{81d}',
    '\u{81e}',
    '\u{81f}',
    '\u{820}',
    '\u{821}',
    '\u{822}',
    '\u{823}',
    '\u{825}',
    '\u{826}',
    '\u{827}',
    '\u{829}',
    '\u{82a}',
    '\u{82b}',
    '\u{82c}',
    '\u{82d}',
    '\u{951}',
    '\u{953}',
    '\u{954}',
    '\u{f82}',
    '\u{f83}',
    '\u{f86}',
    '\u{f87}',
    '\u{135d}',
    '\u{135e}',
    '\u{135f}',
    '\u{17dd}',
    '\u{193a}',
    '\u{1a17}',
    '\u{1a75}',
    '\u{1a76}',
    '\u{1a77}',
    '\u{1a78}',
    '\u{1a79}',
    '\u{1a7a}',
    '\u{1a7b}',
    '\u{1a7c}',
    '\u{1b6b}',
    '\u{1b6d}',
    '\u{1b6e}',
    '\u{1b6f}',
    '\u{1b70}',
    '\u{1b71}',
    '\u{1b72}',
    '\u{1b73}',
    '\u{1cd0}',
    '\u{1cd1}',
    '\u{1cd2}',
    '\u{1cda}',
    '\u{1cdb}',
    '\u{1ce0}',
    '\u{1dc0}',
    '\u{1dc1}',
    '\u{1dc3}',
    '\u{1dc4}',
    '\u{1dc5}',
    '\u{1dc6}',
    '\u{1dc7}',
    '\u{1dc8}',
    '\u{1dc9}',
    '\u{1dcb}',
    '\u{1dcc}',
    '\u{1dd1}',
    '\u{1dd2}',
    '\u{1dd3}',
    '\u{1dd4}',
    '\u{1dd5}',
    '\u{1dd6}',
    '\u{1dd7}',
    '\u{1dd8}',
    '\u{1dd9}',
    '\u{1dda}',
    '\u{1ddb}',
    '\u{1ddc}',
    '\u{1ddd}',
    '\u{1dde}',
    '\u{1ddf}',
    '\u{1de0}',
    '\u{1de1}',
    '\u{1de2}',
    '\u{1de3}',
    '\u{1de4}',
    '\u{1de5}',
    '\u{1de6}',
    '\u{1dfe}',
    '\u{20d0}',
    '\u{20d1}',
    '\u{20d4}',
    '\u{20d5}',
    '\u{20d6}',
    '\u{20d7}',
    '\u{20db}',
    '\u{20dc}',
    '\u{20e1}',
    '\u{20e7}',
    '\u{20e9}',
    '\u{20f0}',
    '\u{2cef}',
    '\u{2cf0}',
    '\u{2cf1}',
    '\u{2de0}',
    '\u{2de1}',
    '\u{2de2}',
    '\u{2de3}',
    '\u{2de4}',
    '\u{2de5}',
    '\u{2de6}',
    '\u{2de7}',
    '\u{2de8}',
    '\u{2de9}',
    '\u{2dea}',
    '\u{2deb}',
    '\u{2dec}',
    '\u{2ded}',
    '\u{2dee}',
    '\u{2def}',
    '\u{2df0}',
    '\u{2df1}',
    '\u{2df2}',
    '\u{2df3}',
    '\u{2df4}',
    '\u{2df5}',
    '\u{2df6}',
    '\u{2df7}',
    '\u{2df8}',
    '\u{2df9}',
    '\u{2dfa}',
    '\u{2dfb}',
    '\u{2dfc}',
    '\u{2dfd}',
    '\u{2dfe}',
    '\u{2dff}',
    '\u{a66f}',
    '\u{a67c}',
    '\u{a67d}',
    '\u{a6f0}',
    '\u{a6f1}',
    '\u{a8e0}',
    '\u{a8e1}',
    '\u{a8e2}',
    '\u{a8e3}',
    '\u{a8e4}',
    '\u{a8e5}',
    '\u{a8e6}',
    '\u{a8e7}',
    '\u{a8e8}',
    '\u{a8e9}',
    '\u{a8ea}',
    '\u{a8eb}',
    '\u{a8ec}',
    '\u{a8ed}',
    '\u{a8ee}',
    '\u{a8ef}',
    '\u{a8f0}',
    '\u{a8f1}',
    '\u{aab0}',
    '\u{aab2}',
    '\u{aab3}',
    '\u{aab7}',
    '\u{aab8}',
    '\u{aabe}',
    '\u{aabf}',
    '\u{aac1}',
    '\u{fe20}',
    '\u{fe21}',
    '\u{fe22}',
    '\u{fe23}',
    '\u{fe24}',
    '\u{fe25}',
    '\u{fe26}',
    '\u{10a0f}',
    '\u{10a38}',
    '\u{1d185}',
    '\u{1d186}',
    '\u{1d187}',
    '\u{1d188}',
    '\u{1d189}',
    '\u{1d1aa}',
    '\u{1d1ab}',
    '\u{1d1ac}',
    '\u{1d1ad}',
    '\u{1d242}',
    '\u{1d243}',
    '\u{1d244}',
];

/// Largest placeholder grid the diacritics can address.
pub const MAX_CELLS: u16 = DIACRITICS.len() as u16;
const PLACEHOLDER: char = '\u{10EEEE}';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Medium {
    /// POSIX shared memory (`t=s`).
    Shm,
    /// Temp file the terminal deletes after reading (`t=t`).
    File,
    /// zlib + base64 inside the escape (`t=d`), works over SSH.
    Direct,
}

impl Medium {
    pub fn parse(s: &str) -> Option<Medium> {
        match s {
            "shm" => Some(Medium::Shm),
            "file" => Some(Medium::File),
            "direct" => Some(Medium::Direct),
            _ => None,
        }
    }

    fn fallback(self) -> Medium {
        match self {
            Medium::Shm => Medium::File,
            _ => Medium::Direct,
        }
    }
}

/// Wrap an escape sequence in tmux DCS passthrough, doubling every ESC.
pub fn wrap_tmux(seq: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(b"\x1bPtmux;");
    for &b in seq {
        if b == 0x1b {
            out.push(0x1b);
        }
        out.push(b);
    }
    out.extend_from_slice(b"\x1b\\");
}

/// Append one graphics command: `ESC _ G <control> [; <payload>] ESC \`.
pub fn command(out: &mut Vec<u8>, control: &str, payload: &[u8], tmux: bool) {
    let mut seq = Vec::with_capacity(control.len() + payload.len() + 6);
    seq.extend_from_slice(b"\x1b_G");
    seq.extend_from_slice(control.as_bytes());
    if !payload.is_empty() {
        seq.push(b';');
        seq.extend_from_slice(payload);
    }
    seq.extend_from_slice(b"\x1b\\");
    if tmux {
        wrap_tmux(&seq, out);
    } else {
        out.extend_from_slice(&seq);
    }
}

/// Delete the image and free its data.
pub fn delete(out: &mut Vec<u8>, id: u32, tmux: bool) {
    command(out, &format!("a=d,d=I,i={id},q=2"), b"", tmux);
}

/// Placeholder text covering `cols` x `rows` cells from the top-left corner.
/// Every cell carries explicit row and column diacritics so partial redraws
/// (tmux, popups) stay correct. `id` must be 1..=255 (256-color foreground).
pub fn placeholders(out: &mut Vec<u8>, id: u32, cols: u16, rows: u16) {
    let (cols, rows) = (cols.min(MAX_CELLS) as usize, rows.min(MAX_CELLS) as usize);
    let mut s = format!("\x1b[0m\x1b[38;5;{id}m");
    for (r, &row) in DIACRITICS[..rows].iter().enumerate() {
        s.push_str(&format!("\x1b[{};1H", r + 1));
        for &col in &DIACRITICS[..cols] {
            s.push(PLACEHOLDER);
            s.push(row);
            s.push(col);
        }
    }
    s.push_str("\x1b[39m");
    out.extend_from_slice(s.as_bytes());
}

const RING: usize = 8;
/// How long a medium may go unread before we give up on it.
const PROBE_TIME: Duration = Duration::from_millis(1200);
/// A slot the terminal never read is reclaimed after this long.
const STALE: Duration = Duration::from_millis(1500);
const CHUNK: usize = 4096;

pub struct Presenter {
    pub id: u32,
    pub medium: Medium,
    tmux: bool,
    forced: bool,
    confirmed: bool,
    /// When the current medium sent its first frame.
    probe_start: Option<Instant>,
    slots: [Option<Instant>; RING],
    next: usize,
    pid: u32,
    tmp: PathBuf,
    rgb: Vec<u8>,
    link: Link,
}

impl Presenter {
    /// `forced` pins the medium (no automatic fallback).
    pub fn new(medium: Medium, forced: bool, tmux: bool) -> Self {
        let pid = std::process::id();
        Presenter {
            id: 16 + pid % 240,
            medium,
            tmux,
            forced,
            confirmed: false,
            probe_start: None,
            slots: [None; RING],
            next: 0,
            pid,
            tmp: std::env::temp_dir(),
            rgb: Vec::new(),
            link: Link::new(),
        }
    }

    pub fn tmux(&self) -> bool {
        self.tmux
    }

    /// The terminal is known to show what we send, so an unchanged frame can
    /// be skipped. Until then every frame goes out so a dead medium falls back.
    pub fn settled(&self) -> bool {
        self.medium == Medium::Direct || self.confirmed || self.forced
    }

    fn shm_name(&self, i: usize) -> CString {
        CString::new(format!("/trex-{}-{i}", self.pid)).expect("no nul in shm name")
    }

    fn file_path(&self, i: usize) -> PathBuf {
        self.tmp.join(format!("tty-graphics-protocol-trex-{}-{i}.rgb", self.pid))
    }

    fn slot_exists(&self, i: usize) -> bool {
        match self.medium {
            Medium::Shm => {
                let name = self.shm_name(i);
                // SAFETY: valid C string; the descriptor is closed right away.
                let fd = unsafe { libc::shm_open(name.as_ptr(), libc::O_RDONLY, 0) };
                if fd >= 0 {
                    unsafe { libc::close(fd) };
                }
                fd >= 0
            }
            Medium::File => self.file_path(i).exists(),
            Medium::Direct => false,
        }
    }

    fn remove_slot(&self, i: usize) {
        match self.medium {
            Medium::Shm => {
                let name = self.shm_name(i);
                // SAFETY: valid C string.
                unsafe { libc::shm_unlink(name.as_ptr()) };
            }
            Medium::File => {
                let _ = fs::remove_file(self.file_path(i));
            }
            Medium::Direct => {}
        }
    }

    fn write_slot(&self, i: usize) -> io::Result<Vec<u8>> {
        match self.medium {
            Medium::Shm => {
                let name = self.shm_name(i);
                write_shm(&name, &self.rgb)?;
                Ok(name.into_bytes())
            }
            Medium::File => {
                let path = self.file_path(i);
                fs::write(&path, &self.rgb)?;
                Ok(path.into_os_string().into_encoded_bytes())
            }
            Medium::Direct => unreachable!("direct frames have no slot"),
        }
    }

    /// Release slots the terminal consumed; reclaim ones it never read.
    fn poll_slots(&mut self) {
        for i in 0..RING {
            let Some(t) = self.slots[i] else { continue };
            if !self.slot_exists(i) {
                self.slots[i] = None;
                self.confirmed = true;
            } else if t.elapsed() > STALE {
                self.remove_slot(i);
                self.slots[i] = None;
            }
        }
    }

    fn fall_back(&mut self) {
        self.clear_slots();
        self.medium = self.medium.fallback();
        self.probe_start = None;
    }

    fn clear_slots(&mut self) {
        for i in 0..RING {
            self.remove_slot(i);
            self.slots[i] = None;
        }
    }

    /// Queue the escapes that replace the image with `cv`, fit to `cols` x
    /// `rows` cells and upscaled by `k`; inline frames go out at 1x unless
    /// `fixed`. False when the frame was dropped.
    pub fn present(
        &mut self,
        out: &mut Vec<u8>,
        cv: &Canvas,
        k: usize,
        fixed: bool,
        cols: u16,
        rows: u16,
    ) -> io::Result<bool> {
        let (cols, rows) = (cols.min(MAX_CELLS), rows.min(MAX_CELLS));
        if self.medium == Medium::Direct {
            return self.present_direct(out, cv, k, fixed, cols, rows);
        }

        cv.scaled_rgb(k, &mut self.rgb);
        let (w, h) = (cv.w as usize * k, cv.h as usize * k);
        self.poll_slots();
        let unread = self.probe_start.is_some_and(|t| t.elapsed() > PROBE_TIME);
        if !self.confirmed && !self.forced && unread {
            self.fall_back();
            return self.present(out, cv, k, fixed, cols, rows);
        }
        let i = self.next;
        if self.slots[i].is_some() {
            return Ok(false); // Terminal is behind; drop this frame.
        }
        let name = match self.write_slot(i) {
            Ok(name) => name,
            Err(_) if !self.forced => {
                self.fall_back();
                return self.present(out, cv, k, fixed, cols, rows);
            }
            Err(e) => return Err(e),
        };
        self.slots[i] = Some(Instant::now());
        self.next = (i + 1) % RING;
        self.probe_start.get_or_insert_with(Instant::now);
        let t = if self.medium == Medium::Shm { 's' } else { 't' };
        let control = format!(
            "a=T,U=1,i={},f=24,s={w},v={h},c={cols},r={rows},q=2,t={t},S={}",
            self.id,
            self.rgb.len()
        );
        command(out, &control, B64.encode(name).as_bytes(), self.tmux);
        Ok(true)
    }

    /// Inline frames, paced by the terminal's answers when it gives them.
    fn present_direct(
        &mut self,
        out: &mut Vec<u8>,
        cv: &Canvas,
        k: usize,
        fixed: bool,
        cols: u16,
        rows: u16,
    ) -> io::Result<bool> {
        let now = Instant::now();
        if !self.link.ready(now) {
            return Ok(false);
        }
        // Kitty's smooth upscale of a 1x frame keeps the link light.
        let k = if fixed { k } else { 1 };
        cv.scaled_rgb(k, &mut self.rgb);
        let (w, h) = (cv.w as usize * k, cv.h as usize * k);
        let answer = self.link.wants_replies();
        if self.link.probe_due(now) {
            command(out, &format!("a=q,i={},s=1,v=1,f=24,t=d", self.probe_id()), b"AAAA", self.tmux);
        }
        let mut z = ZlibEncoder::new(Vec::with_capacity(self.rgb.len() / 8), Compression::fast());
        z.write_all(&self.rgb)?;
        let data = B64.encode(z.finish()?);
        let quiet = if answer { "" } else { ",q=2" };
        let control = format!("a=T,U=1,i={},f=24,s={w},v={h},c={cols},r={rows},t=d,o=z{quiet}", self.id);
        direct_chunks(out, &control, data.as_bytes(), self.tmux, !answer);
        self.link.sent(Instant::now());
        Ok(true)
    }

    /// Image id the round-trip probe asks about; never shown.
    fn probe_id(&self) -> u32 {
        1_000_000 + self.pid
    }

    /// A graphics answer from the terminal for image `id`.
    pub fn answered(&mut self, id: u32, at: Instant) {
        if id == self.id {
            self.link.frame_answered(at);
        } else if id == self.probe_id() {
            self.link.probe_answered(at);
        }
    }

    pub fn link_summary(&self) -> String {
        self.link.summary()
    }
}

/// Longest graphics answer we collect before deciding it is typing.
const MAX_ANSWER: usize = 96;

/// Pulls graphics answers (`ESC _ G <keys> ; <message> ESC \`) out of the
/// key stream. crossterm reads one as Alt+_, plain characters, then Alt+\,
/// so the digits in it would otherwise pick level-up cards.
#[derive(Default)]
pub struct AnswerFilter {
    open: bool,
    text: String,
    held: Vec<Event>,
}

impl AnswerFilter {
    /// Take one event. Events that are not part of an answer land in `pass`;
    /// a finished answer returns its image id.
    pub fn feed(&mut self, ev: Event, pass: &mut Vec<Event>) -> Option<u32> {
        let key = match &ev {
            Event::Key(k) if k.kind == KeyEventKind::Press => Some(*k),
            _ => None,
        };
        let alt =
            |k: KeyEvent, c: char| k.code == KeyCode::Char(c) && k.modifiers.contains(KeyModifiers::ALT);
        if !self.open {
            if key.is_some_and(|k| alt(k, '_')) {
                self.open = true;
                self.held.push(ev);
            } else {
                pass.push(ev);
            }
            return None;
        }
        match key {
            Some(k) if alt(k, '\\') => {
                self.open = false;
                let id = answer_id(&self.text);
                self.text.clear();
                if id.is_some() {
                    self.held.clear();
                } else {
                    pass.append(&mut self.held);
                    pass.push(ev);
                }
                id
            }
            Some(KeyEvent { code: KeyCode::Char(c), modifiers, .. })
                if !modifiers.intersects(KeyModifiers::ALT | KeyModifiers::CONTROL)
                    && (!self.text.is_empty() || c == 'G')
                    && self.text.len() < MAX_ANSWER =>
            {
                self.text.push(c);
                self.held.push(ev);
                None
            }
            _ => {
                self.open = false;
                self.text.clear();
                pass.append(&mut self.held);
                self.feed(ev, pass)
            }
        }
    }
}

/// Image id in the body of a graphics answer, `G<keys>;<message>`.
fn answer_id(text: &str) -> Option<u32> {
    let (keys, _) = text.strip_prefix('G')?.split_once(';')?;
    keys.split(',').find_map(|kv| kv.strip_prefix("i=")).and_then(|v| v.parse().ok())
}

/// Frames the terminal never picked up would outlive us (shared memory
/// until reboot), so they go on every exit path, including errors and panics.
impl Drop for Presenter {
    fn drop(&mut self) {
        if self.medium != Medium::Direct {
            self.clear_slots();
        }
    }
}

/// Split base64 `data` into escapes of at most 4096 payload bytes; only the
/// first carries `control`, the rest carry `m` and, when `quiet`, `q`.
pub fn direct_chunks(out: &mut Vec<u8>, control: &str, data: &[u8], tmux: bool, quiet: bool) {
    let n = data.len().div_ceil(CHUNK);
    let q = if quiet { ",q=2" } else { "" };
    for (i, chunk) in data.chunks(CHUNK).enumerate() {
        let more = u8::from(i + 1 < n);
        let ctl = if i == 0 { format!("{control},m={more}") } else { format!("m={more}{q}") };
        command(out, &ctl, chunk, tmux);
    }
}

fn write_shm(name: &CString, data: &[u8]) -> io::Result<()> {
    // SAFETY: plain POSIX calls on a descriptor we own; the mapping is exactly
    // `data.len()` bytes and unmapped before return.
    unsafe {
        // The slot is free, so an object by this name is left over from a
        // crashed process that had our pid.
        libc::shm_unlink(name.as_ptr());
        let fd =
            libc::shm_open(name.as_ptr(), libc::O_CREAT | libc::O_EXCL | libc::O_RDWR, 0o600 as libc::c_uint);
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let fail = |fd: libc::c_int| {
            let e = io::Error::last_os_error();
            libc::close(fd);
            libc::shm_unlink(name.as_ptr());
            Err(e)
        };
        if libc::ftruncate(fd, data.len() as libc::off_t) != 0 {
            return fail(fd);
        }
        let ptr = libc::mmap(
            std::ptr::null_mut(),
            data.len(),
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        );
        if ptr == libc::MAP_FAILED {
            return fail(fd);
        }
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast::<u8>(), data.len());
        libc::munmap(ptr, data.len());
        libc::close(fd);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmux_wrap_doubles_escapes() {
        let mut out = Vec::new();
        wrap_tmux(b"\x1b_Ga=d\x1b\\", &mut out);
        assert_eq!(out, b"\x1bPtmux;\x1b\x1b_Ga=d\x1b\x1b\\\x1b\\");
    }

    #[test]
    fn placeholder_cells_carry_row_and_column() {
        let mut out = Vec::new();
        placeholders(&mut out, 42, 2, 2);
        let s = String::from_utf8(out).unwrap();
        assert!(s.starts_with("\x1b[0m\x1b[38;5;42m\x1b[1;1H"));
        assert!(s.contains("\u{10EEEE}\u{305}\u{305}\u{10EEEE}\u{305}\u{30d}\x1b[2;1H"));
        assert!(s.contains("\u{10EEEE}\u{30d}\u{305}\u{10EEEE}\u{30d}\u{30d}\x1b[39m"));
        assert_eq!(DIACRITICS[296], '\u{1d244}');
    }

    #[test]
    fn direct_chunks_split_on_4096_with_more_flags() {
        let data = vec![b'A'; CHUNK * 2 + 10];
        let mut out = Vec::new();
        direct_chunks(&mut out, "a=T,i=7", &data, false, true);
        let s = String::from_utf8(out).unwrap();
        let parts: Vec<&str> = s.split("\x1b\\").filter(|p| !p.is_empty()).collect();
        assert_eq!(parts.len(), 3);
        assert!(parts[0].starts_with("\x1b_Ga=T,i=7,m=1;"));
        assert!(parts[1].starts_with("\x1b_Gm=1,q=2;"));
        assert!(parts[2].starts_with("\x1b_Gm=0,q=2;"));
        assert_eq!(parts[0].len(), "\x1b_Ga=T,i=7,m=1;".len() + CHUNK);
    }

    fn key(c: char, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), modifiers))
    }

    /// Events crossterm makes of `ESC _ <text> ESC \`.
    fn answer(text: &str) -> Vec<Event> {
        let mut evs = vec![key('_', KeyModifiers::ALT)];
        for c in text.chars() {
            let m = if c.is_ascii_uppercase() { KeyModifiers::SHIFT } else { KeyModifiers::NONE };
            evs.push(key(c, m));
        }
        evs.push(key('\\', KeyModifiers::ALT));
        evs
    }

    #[test]
    fn answers_leave_the_key_stream() {
        let mut f = AnswerFilter::default();
        let mut pass = Vec::new();
        let mut ids = Vec::new();
        let mut evs = vec![key('w', KeyModifiers::NONE)];
        evs.extend(answer("Gi=31;OK"));
        evs.push(key('1', KeyModifiers::NONE));
        evs.extend(answer("Gi=7;ENODATA:no data"));
        for ev in evs {
            ids.extend(f.feed(ev, &mut pass));
        }
        assert_eq!(ids, [31, 7]);
        assert_eq!(pass, [key('w', KeyModifiers::NONE), key('1', KeyModifiers::NONE)]);
    }

    #[test]
    fn typing_that_is_not_an_answer_passes_through() {
        let mut f = AnswerFilter::default();
        let mut pass = Vec::new();
        let evs = [key('_', KeyModifiers::ALT), key('a', KeyModifiers::NONE), key('_', KeyModifiers::ALT)];
        for ev in evs {
            assert_eq!(f.feed(ev, &mut pass), None);
        }
        assert_eq!(pass, evs[..2]);
    }
}
