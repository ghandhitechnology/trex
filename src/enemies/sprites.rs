//! Enemy art. Enemies face right; the renderer flips them by movement.

use crate::render::sprite::SpriteDef;

pub const SPRITES: &[SpriteDef] = &[
    SpriteDef {
        name: "grub",
        outline: true,
        frames: &[
            &[
                "....zzzz....", //
                "..zzxxxxzz..", //
                ".zxxYYxxxxz.", //
                ".zxYYxxxxxz.", //
                "zxxxxxkxkxxz", //
                "zxxxxxkxkxxz", //
                "zZxxxxxxxxZz", //
                ".zZZxxxxZZz.", //
                "..zzZZZZzz..", //
            ],
            &[
                "............", //
                "...zzzzzz...", //
                ".zzxxxxxxzz.", //
                "zxxYYxxxxxxz", //
                "zxYYxxkxkxxz", //
                "zxxxxxkxkxxz", //
                "zZxxxxxxxxZz", //
                ".zZZZxxZZZz.", //
                "..zzzzzzzz..", //
            ],
        ],
    },
    SpriteDef {
        name: "wisp",
        outline: true,
        frames: &[
            &[
                ".....h.....", //
                "....hH.....", //
                "...hHh..h..", //
                "..hHHhhhH..", //
                "..hHHHHHh..", //
                ".hHHwHHwHh.", //
                ".hHHkHHkHh.", //
                ".hHHHHHHHh.", //
                ".PhHHHHHhP.", //
                "..PhhhhhP..", //
                "..P.PhP.P..", //
                ".....P.....", //
            ],
            &[
                "......h....", //
                ".h...Hh....", //
                "..h.hHh.h..", //
                "..hhHHhhH..", //
                "..hHHHHHh..", //
                ".hHHwHHwHh.", //
                ".hHHkHHkHh.", //
                ".hHHHHHHHh.", //
                ".PhHHHHHhP.", //
                "..PhhhhhP..", //
                "...PPhPP...", //
                "...P...P...", //
            ],
        ],
    },
    SpriteDef {
        name: "brute",
        outline: true,
        frames: &[
            &[
                "....ssssss....", //
                "...seeeeeess..", //
                "..seevvvvvvss.", //
                "..svoovvoovvs.", //
                "..svvvvvvvvvs.", //
                ".sssvkkkkvvsss", //
                "seevsvvvvvsvee", //
                "sevvsssssssvve", //
                "svvvsvvvvvsvvv", //
                "ssvssvvovvssvs", //
                ".ss.svvvvvs.s.", //
                "....sss.sss...", //
                "...ssss.ssss..", //
            ],
            &[
                "....ssssss....", //
                "...seeeeeess..", //
                "..seevvvvvvss.", //
                "..svoovvoovvs.", //
                "..svvvvvvvvvs.", //
                ".sssvkkkkvvsss", //
                "seevsvvvvvsvee", //
                "sevvsssssssvve", //
                "svvvsvvvvvsvvv", //
                "ssvssvvovvssvs", //
                ".ss.svvvvvs.s.", //
                "...sss...sss..", //
                "..ssss...ssss.", //
            ],
        ],
    },
    SpriteDef {
        name: "spit",
        outline: true,
        frames: &[&[
            ".l.", //
            "lLl", //
            ".g.", //
        ]],
    },
];
