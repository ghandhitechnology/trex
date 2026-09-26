//! Item icons (12x12 art, 14x14 with outline) and item projectiles.

use crate::render::sprite::SpriteDef;

pub const SPRITES: &[SpriteDef] = &[
    SpriteDef {
        name: "spark",
        outline: true,
        frames: &[
            &[
                ".I.", //
                "IiI", //
                ".I.", //
            ],
            &[
                "i.i", //
                ".I.", //
                "i.i", //
            ],
        ],
    },
    SpriteDef {
        name: "hot_lead",
        outline: true,
        frames: &[&[
            ".....r......", //
            "....ro......", //
            "....roo..r..", //
            "...roao..ro.", //
            "..roaaorroo.", //
            "..roayaooao.", //
            ".roaayyaaaor", //
            ".roayYYyaaor", //
            ".roayYYYyaor", //
            "..roayYyaor.", //
            "...rooaaor..", //
            "....rrrrr...", //
        ]],
    },
    SpriteDef {
        name: "twin_barrel",
        outline: true,
        frames: &[&[
            "............", //
            "sssssssss...", //
            "peeeeeeeesk.", //
            "evvvvvvvvsk.", //
            "sssssssss...", //
            "..zZz.......", //
            "sssssssss...", //
            "peeeeeeeesk.", //
            "evvvvvvvvsk.", //
            "sssssssss...", //
            "..zZz.......", //
            "..zzz.......", //
        ]],
    },
    SpriteDef {
        name: "rubber_ball",
        outline: true,
        frames: &[&[
            "....hhhh....", //
            "..hhHHhhhh..", //
            ".hHHwHhhhhP.", //
            ".hHwHhhhhhP.", //
            "hhHHhhhhhhPP", //
            "hhhhhhhhhPPP", //
            "hhhhhhhhhPPP", //
            "hhhhhhhhPPPu", //
            ".hhhhhhPPPu.", //
            ".PPhhPPPPuu.", //
            "..PPPPPPuu..", //
            "....uuuu....", //
        ]],
    },
    SpriteDef {
        name: "drill_bit",
        outline: true,
        frames: &[&[
            "..........pw", //
            ".........pwe", //
            "........pwe.", //
            ".......pwve.", //
            "......ewve..", //
            ".....pwve...", //
            "....ewve....", //
            "...zzve.....", //
            "..zZZz......", //
            ".zZZz.......", //
            "zZZz........", //
            "zzz.........", //
        ]],
    },
    SpriteDef {
        name: "powder_keg",
        outline: true,
        frames: &[&[
            ".......y.Y..", //
            "........a...", //
            ".......k....", //
            "...zzzzkzz..", //
            "..zZZZZZZZz.", //
            "..sssssssss.", //
            "..zZZxZZZZz.", //
            "..zZxZZZZZz.", //
            "..sssssssss.", //
            "..zZZZZZZZz.", //
            "...zzzzzzz..", //
            "............", //
        ]],
    },
    SpriteDef {
        name: "spark_plug",
        outline: true,
        frames: &[&[
            ".....ss.....", //
            "....seps....", //
            "....sees....", //
            "...wwwwww...", //
            "...wppppw...", //
            "...wppppw...", //
            "...wwwwww...", //
            "....svvs....", //
            "....svvs....", //
            ".....ss.....", //
            ".....i......", //
            "....iI......", //
        ]],
    },
    SpriteDef {
        name: "black_coffee",
        outline: true,
        frames: &[&[
            "...p..p.....", //
            "....p..p....", //
            "...p..p.....", //
            "............", //
            ".wwwwwww....", //
            ".wzzzzzwww..", //
            ".wzkkkzw.w..", //
            ".wzkkkzw.w..", //
            ".wzzzzzwww..", //
            ".wwwwwww....", //
            "..pppppp....", //
            "............", //
        ]],
    },
    SpriteDef {
        name: "magnet",
        outline: true,
        frames: &[&[
            "............", //
            "...rrrrrr...", //
            "..rrHHHrrr..", //
            ".rrH....rrr.", //
            ".rr......rr.", //
            ".rr......rr.", //
            ".rr......rr.", //
            ".pp......pp.", //
            ".ww......ww.", //
            ".pp......pp.", //
            "............", //
            "............", //
        ]],
    },
    SpriteDef {
        name: "heart_jar",
        outline: true,
        frames: &[&[
            "...ssssss...", //
            "...eeeeee...", //
            "..pI....Ip..", //
            ".p.I....I.p.", //
            ".p..rr.rr.p.", //
            ".p.rHrrrrcp.", //
            ".p.rrrrrrcp.", //
            ".p..rrrrc.p.", //
            ".p...rrc..p.", //
            ".p....c...p.", //
            "..pppppppp..", //
            "............", //
        ]],
    },
    SpriteDef {
        name: "static_coil",
        outline: true,
        frames: &[&[
            "....bbbb....", //
            "...bBBBBb...", //
            "....bbbb....", //
            "...bBBBBb...", //
            "....bbbb....", //
            "...bBBBBb...", //
            "....bbbb....", //
            "...bBBBBb...", //
            "....bbbb....", //
            ".....iI.....", //
            "....iIi.....", //
            ".....i......", //
        ]],
    },
];
