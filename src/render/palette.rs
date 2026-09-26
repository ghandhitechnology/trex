//! The one fixed palette ("Tarpit 32"). Every pixel the game draws comes from here,
//! except blended overlays.
//!
//! Sprite strings use one character per pixel. `.` is transparent; every other
//! character maps to a palette color through [`CHARS`]:
//!
//! | ramp    | dark → light                                   |
//! |---------|------------------------------------------------|
//! | neutral | `k` ink `n` night `d` dusk `s` slate `v` mauve `e` haze `p` fog `w` bone |
//! | warm    | `m` maroon `c` blood `r` red `o` ember `a` amber `y` gold `Y` cream |
//! | green   | `t` deep `f` moss `g` leaf `l` lime `L` sprout |
//! | blue    | `N` navy `b` blue `B` sky `i` cyan `I` ice |
//! | purple  | `u` plum `P` grape `h` pink `H` blush |
//! | earth   | `z` umber `Z` clay `x` sand |

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn hex(rgb: u32) -> Self {
        Self { r: (rgb >> 16) as u8, g: (rgb >> 8) as u8, b: rgb as u8, a: 255 }
    }

    /// Linear mix toward `o` by `t` in 0..=255.
    pub fn mix(self, o: Color, t: u8) -> Color {
        let t = u16::from(t);
        let f = |a: u8, b: u8| ((u16::from(a) * (255 - t) + u16::from(b) * t) / 255) as u8;
        Color { r: f(self.r, o.r), g: f(self.g, o.g), b: f(self.b, o.b), a: 255 }
    }
}

pub const INK: Color = Color::hex(0x0f0b18);
pub const NIGHT: Color = Color::hex(0x1d1629);
pub const DUSK: Color = Color::hex(0x2d2340);
pub const SLATE: Color = Color::hex(0x45395c);
pub const MAUVE: Color = Color::hex(0x675a7c);
pub const HAZE: Color = Color::hex(0x948aa6);
pub const FOG: Color = Color::hex(0xc9c2d4);
pub const BONE: Color = Color::hex(0xf5efe0);

pub const MAROON: Color = Color::hex(0x4a1530);
pub const BLOOD: Color = Color::hex(0x86203a);
pub const RED: Color = Color::hex(0xcc3a3f);
pub const EMBER: Color = Color::hex(0xef6b3a);
pub const AMBER: Color = Color::hex(0xf7a041);
pub const GOLD: Color = Color::hex(0xffd25e);
pub const CREAM: Color = Color::hex(0xfff4b0);

pub const DEEP: Color = Color::hex(0x0f3134);
pub const MOSS: Color = Color::hex(0x1b5a45);
pub const LEAF: Color = Color::hex(0x2e904f);
pub const LIME: Color = Color::hex(0x7cc84b);
pub const SPROUT: Color = Color::hex(0xcdeb72);

pub const NAVY: Color = Color::hex(0x1a2a5e);
pub const BLUE: Color = Color::hex(0x2a5aa8);
pub const SKY: Color = Color::hex(0x3e9ce0);
pub const CYAN: Color = Color::hex(0x74dcee);
pub const ICE: Color = Color::hex(0xd2fbf6);

pub const PLUM: Color = Color::hex(0x3e1b58);
pub const GRAPE: Color = Color::hex(0x7a36a0);
pub const PINK: Color = Color::hex(0xda4f9e);
pub const BLUSH: Color = Color::hex(0xff99c4);

pub const UMBER: Color = Color::hex(0x52301f);
pub const CLAY: Color = Color::hex(0x955c38);
pub const SAND: Color = Color::hex(0xdca56f);

pub const PALETTE: [Color; 32] = [
    INK, NIGHT, DUSK, SLATE, MAUVE, HAZE, FOG, BONE, //
    MAROON, BLOOD, RED, EMBER, AMBER, GOLD, CREAM, //
    DEEP, MOSS, LEAF, LIME, SPROUT, //
    NAVY, BLUE, SKY, CYAN, ICE, //
    PLUM, GRAPE, PINK, BLUSH, //
    UMBER, CLAY, SAND,
];

/// Sprite character for each palette index, in `PALETTE` order.
pub const CHARS: &str = "kndsvepwmcroayYtfglLNbBiIuPhHzZx";

/// Palette index used for transparent sprite pixels.
pub const CLEAR: u8 = u8::MAX;

/// Palette index for a sprite character. `None` for unknown characters.
pub fn index_of(c: char) -> Option<u8> {
    if c == '.' {
        return Some(CLEAR);
    }
    CHARS.find(c).map(|i| i as u8)
}

pub fn color(index: u8) -> Color {
    PALETTE[index as usize]
}
