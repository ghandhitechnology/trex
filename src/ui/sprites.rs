//! HUD and menu art.

use crate::render::sprite::SpriteDef;

pub const SPRITES: &[SpriteDef] = &[
    SpriteDef {
        name: "heart_full",
        outline: true,
        frames: &[&[
            ".rr.rr.", //
            "rHrrrrc", //
            "rrrrrrc", //
            ".rrrrc.", //
            "..rcc..", //
            "...c...", //
        ]],
    },
    SpriteDef {
        name: "heart_half",
        outline: true,
        frames: &[&[
            ".rr.dd.", //
            "rHrdddd", //
            "rrrdddd", //
            ".rrddd.", //
            "..rdd..", //
            "...d...", //
        ]],
    },
    SpriteDef {
        name: "heart_empty",
        outline: true,
        frames: &[&[
            ".dd.dd.", //
            "dsddddd", //
            "ddddddd", //
            ".ddddd.", //
            "..ddd..", //
            "...d...", //
        ]],
    },
    SpriteDef {
        name: "bone",
        outline: true,
        frames: &[&[
            "pw...wp", //
            ".wwwww.", //
            "pw...wp", //
        ]],
    },
    SpriteDef {
        name: "skull",
        outline: true,
        frames: &[&[
            ".wwww.", //
            "wwwwww", //
            "wkwwkw", //
            "wwwwww", //
            ".wkkw.", //
            ".p.p..", //
        ]],
    },
    SpriteDef {
        name: "logo",
        outline: true,
        frames: &[&[
            "YYYYYYYYYY..YYYYYYYYY...YYYYYYYYY..YYYY....YYYY.",
            "yyyyyyyyyym.yyyyyyyyyy..yyyyyyyyym.yyyym...yyyym",
            "yyyyyyyyyym.yyyymmyyyym.yyyymmmmmm..yyyy..yyyymm",
            ".mmaaaammmm.aaaam..aaam.aaaam........aaaaaaaamm.",
            "...aaaam....aaaam.aaaam.aaaam.........aaaaaamm..",
            "...aaaam....aaaaaaaaamm.aaaaaaaa.......aaaamm...",
            "...oooom....oooooooomm..oooooooom.....oooooo....",
            "...oooom....oooomoooo...oooommmmm....oooooooo...",
            "...oooom....oooom.oooo..oooom.......oooommoooo..",
            "...rrrrm....rrrrm.rrrrm.rrrrm......rrrrmm..rrrr.",
            "...rrrrm....rrrrm..rrrm.rrrrrrrrr..rrrrm...rrrrm",
            "...ccccm....ccccm..cccm.cccccccccm.ccccm...ccccm",
            "....mmmm.....mmmm...mmm..mmmmmmmmm..mmmm....mmmm",
        ]],
    },
    SpriteDef {
        name: "cursor",
        outline: true,
        frames: &[&[
            "..y..", //
            ".yyy.", //
            "yyyyy", //
        ]],
    },
];
