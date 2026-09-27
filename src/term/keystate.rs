//! Physical key state from the OS, for terminals that never report key release.
//!
//! macOS answers whether a key is down right now. The answer describes this
//! machine's keyboard, so over SSH it says nothing about the player's keys;
//! `Input` only trusts it for a hold whose press the OS saw as down.

use crossterm::event::KeyCode;

/// Whether `code` is physically down, or `None` when the OS cannot tell.
#[cfg(target_os = "macos")]
pub fn down(code: KeyCode) -> Option<bool> {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceKeyState(state: i32, key: u16) -> bool;
    }
    const HID_SYSTEM_STATE: i32 = 1;
    // ANSI virtual key codes (key positions, as on a QWERTY layout).
    let key = match code {
        KeyCode::Char(c) => match c.to_ascii_lowercase() {
            'a' => 0,
            's' => 1,
            'd' => 2,
            'w' => 13,
            _ => return None,
        },
        KeyCode::Left => 123,
        KeyCode::Right => 124,
        KeyCode::Down => 125,
        KeyCode::Up => 126,
        _ => return None,
    };
    // SAFETY: a plain query with no pointers; any key code is accepted.
    Some(unsafe { CGEventSourceKeyState(HID_SYSTEM_STATE, key) })
}

#[cfg(not(target_os = "macos"))]
pub fn down(_: KeyCode) -> Option<bool> {
    None
}
