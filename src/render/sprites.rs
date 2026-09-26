//! Shared world art: XP gems, the spawn warning, and arena floor props.
//! Props have no outline and stay low-contrast so they read as ground.

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
    SpriteDef {
        name: "p_ribs",
        outline: false,
        frames: &[&[
            ".vveeeeeevv.", //
            "v..e..e..e.v", //
            "...e..e..e..", //
            "...v..e..v..", //
            "......v.....", //
        ]],
    },
    SpriteDef {
        name: "p_skull",
        outline: false,
        frames: &[&[
            "...vveee..", //
            "..veeppee.", //
            ".veendpeev", //
            "veeeeeeeee", //
            "vdevevevd.", //
            ".d.d.d.d..", //
        ]],
    },
    SpriteDef {
        name: "p_bone",
        outline: false,
        frames: &[&[
            "e.....e", //
            ".veeee.", //
            "v.....v", //
        ]],
    },
    SpriteDef {
        name: "p_rock",
        outline: false,
        frames: &[&[
            "..ssd..", //
            ".svsdd.", //
            "sssdddn", //
            "dddnnn.", //
            ".nnnn..", //
        ]],
    },
    SpriteDef {
        name: "p_tuft_dim",
        outline: false,
        frames: &[&[
            "..f..", //
            "f.t.f", //
            ".ttt.", //
        ]],
    },
    SpriteDef {
        name: "p_tuft",
        outline: false,
        frames: &[&[
            ".f...", //
            ".f.g.", //
            "f.fg.", //
            ".tft.", //
        ]],
    },
    SpriteDef {
        name: "p_fern",
        outline: false,
        frames: &[&[
            "....f....", //
            ".f..f..f.", //
            "..f.f.f..", //
            "t..fgf..t", //
            ".tt.f.tt.", //
            "...tft...", //
            "....t....", //
        ]],
    },
    SpriteDef {
        name: "p_bloom",
        outline: false,
        frames: &[&[
            ".h...", //
            "hHh.P", //
            ".h.PH", //
            ".f..f", //
            "tf.f.", //
        ]],
    },
    SpriteDef {
        name: "p_mossrock",
        outline: false,
        frames: &[&[
            "..ffg...", //
            ".fgfvs..", //
            "fvvsvss.", //
            "vvsssdd.", //
            ".ssddd..", //
            "..ttt...", //
        ]],
    },
    SpriteDef {
        name: "p_obsidian",
        outline: false,
        frames: &[&[
            "...u....", //
            "..unn.u.", //
            ".nuPnunn", //
            "nnuunnun", //
            "knnunnnk", //
            ".kkkkkk.", //
        ]],
    },
    SpriteDef {
        name: "p_ashpile",
        outline: false,
        frames: &[&[
            "...sd...", //
            ".dsddnd.", //
            "dndnnnnk", //
            ".nkcknn.", //
        ]],
    },
    SpriteDef {
        name: "p_basalt",
        outline: false,
        frames: &[&[
            "..ds...", //
            ".dsdn..", //
            "ddnnnk.", //
            "nnnnkkk", //
            ".kkkk..", //
        ]],
    },
];
