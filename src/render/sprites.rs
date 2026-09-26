//! Shared world art: XP gems and the spawn warning.

use super::sprite::SpriteDef;

pub const SPRITES: &[SpriteDef] = &[
    SpriteDef {
        name: "gem",
        outline: true,
        frames: &[
            &[
                ".I.", //
                "iIi", //
                "Bii", //
                ".B.", //
            ],
            &[
                ".i.", //
                "iiI", //
                "BiI", //
                ".B.", //
            ],
        ],
    },
    SpriteDef {
        name: "gem_big",
        outline: true,
        frames: &[
            &[
                "..I..", //
                ".IiB.", //
                "IiiiB", //
                "iiiBB", //
                ".iBb.", //
                "..b..", //
            ],
            &[
                "..i..", //
                ".iIB.", //
                "iiIiB", //
                "iiiIB", //
                ".iBb.", //
                "..b..", //
            ],
        ],
    },
    SpriteDef {
        name: "warn",
        outline: false,
        frames: &[
            &[
                "r.....r", //
                ".c...c.", //
                "..r.r..", //
                "...o...", //
                "..r.r..", //
                ".c...c.", //
                "r.....r", //
            ],
            &[
                ".......", //
                ".r...r.", //
                "..c.c..", //
                "...r...", //
                "..c.c..", //
                ".r...r.", //
                ".......", //
            ],
        ],
    },
];
