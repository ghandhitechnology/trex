//! Character art (walk and idle frames), weapon projectiles, and meta menu icons.
//! Characters face right; the renderer flips them.

use crate::render::sprite::SpriteDef;

pub const SPRITES: &[SpriteDef] = &[
    SpriteDef {
        name: "rex",
        outline: true,
        frames: &[
            &[
                ".......ggggg..", //
                "......glllllg.", //
                "......gllwklgg", //
                "......glllgggg", //
                "......gggggggg", //
                "......gkwkwkk.", //
                "...g..gffffff.", //
                "..gg.gggxxg...", //
                ".ggggggggxxgg.", //
                "gg..gfgggxx...", //
                "....ffgggg....", //
                "....ff..gg....", //
                "...fff..ggg...", //
                "..............", //
            ],
            &[
                ".......ggggg..", //
                "......glllllg.", //
                "......gllwklgg", //
                "......glllgggg", //
                "......gggggggg", //
                "......gkwkwkk.", //
                "......gffffff.", //
                "...g.gggxxg...", //
                ".ggggggggxxgg.", //
                "gg..gfgggxx...", //
                "....ffgggg....", //
                ".....fggg.....", //
                ".....ffggg....", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "rex_idle",
        outline: true,
        frames: &[
            &[
                ".......ggggg..", //
                "......glllllg.", //
                "......gllwklgg", //
                "......glllgggg", //
                "......gggggggg", //
                "......gkwkwkk.", //
                "...g..gffffff.", //
                "..gg.gggxxg...", //
                ".ggggggggxxgg.", //
                "gg..gfgggxx...", //
                "....ffgggg....", //
                "....ff..gg....", //
                "...fff..ggg...", //
                "..............", //
            ],
            &[
                "..............", //
                ".......ggggg..", //
                "......glllllg.", //
                "......glllllgg", //
                "......glllgggg", //
                "......gggggggg", //
                "......gkwkwkk.", //
                "...g..gffffff.", //
                "..gg.gggxxg...", //
                ".ggggggggxxgg.", //
                "gg..gfgggxx...", //
                "....ff..gg....", //
                "...fff..ggg...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "trike",
        outline: true,
        frames: &[
            &[
                "..............", //
                "..............", //
                "........rr....", //
                ".......rYrr...", //
                ".......rrrr.w.", //
                "...aaa.rrrr.ww", //
                ".aaaaaarraaaw.", //
                "aooaaaaaraakaa", //
                ".oaaaaaaaaaaaw", //
                "..oaaaaaaaaYY.", //
                "..oYYYYYYao...", //
                "..oo.oo..oo...", //
                "..zz.zz..zz...", //
                "..............", //
            ],
            &[
                "..............", //
                "..............", //
                "........rr....", //
                ".......rYrr...", //
                ".......rrrr.w.", //
                "...aaa.rrrr.ww", //
                ".aaaaaarraaaw.", //
                "aooaaaaaraakaa", //
                ".oaaaaaaaaaaaw", //
                "..oaaaaaaaaYY.", //
                "..oYYYYYYao...", //
                "...oo.oo..oo..", //
                "...zz.zz..zz..", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "trike_idle",
        outline: true,
        frames: &[
            &[
                "..............", //
                "..............", //
                "........rr....", //
                ".......rYrr...", //
                ".......rrrr.w.", //
                "...aaa.rrrr.ww", //
                ".aaaaaarraaaw.", //
                "aooaaaaaraakaa", //
                ".oaaaaaaaaaaaw", //
                "..oaaaaaaaaYY.", //
                "..oYYYYYYao...", //
                "..oo..oo.oo...", //
                "..zz..zz.zz...", //
                "..............", //
            ],
            &[
                "..............", //
                "..............", //
                "..............", //
                "........rr....", //
                ".......rYrr...", //
                ".......rrrr.w.", //
                "...aaa.rrrr.ww", //
                ".aaaaaarraaaw.", //
                "aooaaaaaraaoaa", //
                ".oaaaaaaaaaaaw", //
                "..oaaaaaaaaYY.", //
                "..oo..oo.oo...", //
                "..zz..zz.zz...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "ptera",
        outline: true,
        frames: &[
            &[
                "..bb..........", //
                "..bBb.........", //
                "...bBb....oo..", //
                "...bBBb..oBBo.", //
                "....bBBbbBBkYY", //
                ".....bBBBBBYY.", //
                "....bBBBBBb...", //
                "..bbBBIIBBb...", //
                ".bb...bIIb....", //
                "......bbb.....", //
                "..............", //
                ".......y.y....", //
                "..............", //
                "..............", //
            ],
            &[
                "..............", //
                "..............", //
                "..........oo..", //
                ".........oBBo.", //
                "....bbbbbBBkYY", //
                "..bBBBBBBBBYY.", //
                ".bBBBBBBBBb...", //
                "bBBbbBIIBBb...", //
                "Bb..bbbIIb....", //
                "b....bbbbb....", //
                "..............", //
                ".......y.y....", //
                "..............", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "ptera_idle",
        outline: true,
        frames: &[
            &[
                "..bb..........", //
                "..bBb.........", //
                "...bBb....oo..", //
                "...bBBb..oBBo.", //
                "....bBBbbBBkYY", //
                ".....bBBBBBYY.", //
                "....bBBBBBb...", //
                "..bbBBIIBBb...", //
                ".bb...bIIb....", //
                "......bbb.....", //
                "..............", //
                ".......y.y....", //
                "..............", //
                "..............", //
            ],
            &[
                "..............", //
                "..............", //
                "..........oo..", //
                ".........oBBo.", //
                "....bbbbbBBkYY", //
                "..bBBBBBBBBYY.", //
                ".bBBBBBBBBb...", //
                "bBBbbBIIBBb...", //
                "Bb..bbbIIb....", //
                "b....bbbbb....", //
                "..............", //
                ".......y.y....", //
                "..............", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "raptor",
        outline: true,
        frames: &[
            &[
                "..............", //
                "..............", //
                "..........rrr.", //
                ".........rrwkr", //
                ".........rrrrr", //
                "cc.......rYYY.", //
                ".rrc....rrr...", //
                "..rrrcrrrr....", //
                "...rrrrcrrY...", //
                ".....rYYYr.Y..", //
                "......rrc.....", //
                "......c..c....", //
                ".....cc...cc..", //
                ".....w.....w..", //
            ],
            &[
                "..............", //
                "..............", //
                "..........rrr.", //
                ".........rrwkr", //
                ".........rrrrr", //
                ".........rYYY.", //
                "crrc....rrr...", //
                ".rrrrcrrrr....", //
                "...rrrrcrrY...", //
                ".....rYYYr.Y..", //
                "......rrc.....", //
                "......cc......", //
                "......ccc.....", //
                "......w.w.....", //
            ],
        ],
    },
    SpriteDef {
        name: "raptor_idle",
        outline: true,
        frames: &[
            &[
                "..............", //
                "..............", //
                "..........rrr.", //
                ".........rrwkr", //
                ".........rrrrr", //
                "cc.......rYYY.", //
                ".rrc....rrr...", //
                "..rrrcrrrr....", //
                "...rrrrcrrY...", //
                ".....rYYYr.Y..", //
                "......rrc.....", //
                "......c..c....", //
                ".....cc..cc...", //
                ".....w...w....", //
            ],
            &[
                "..............", //
                "..............", //
                "..............", //
                "..........rrr.", //
                ".........rrrcr", //
                ".........rrrrr", //
                "cc.......rYYY.", //
                ".rrc....rrr...", //
                "..rrrcrrrr....", //
                "...rrrrcrrY...", //
                ".....rYYYr.Y..", //
                "......c..c....", //
                ".....cc..cc...", //
                ".....w...w....", //
            ],
        ],
    },
    SpriteDef {
        name: "stego",
        outline: true,
        frames: &[
            &[
                "..............", //
                "..............", //
                ".....h........", //
                "...h.hH.h.....", //
                "..hH.hHhhH....", //
                "..PPPPPPPPP...", //
                ".PPPPPPPPPPPP.", //
                "PPuPPPPPPPPkPP", //
                "P.uPPPPPPPPPPP", //
                "..uPHHHHHHPu..", //
                "..uuPPPPPPuu..", //
                "..uu.uu.uu.uu.", //
                "..nn.nn.nn.nn.", //
                "..............", //
            ],
            &[
                "..............", //
                "..............", //
                ".....h........", //
                "...h.hH.h.....", //
                "..hH.hHhhH....", //
                "..PPPPPPPPP...", //
                ".PPPPPPPPPPPP.", //
                "PPuPPPPPPPPkPP", //
                "P.uPPPPPPPPPPP", //
                "..uPHHHHHHPu..", //
                "..uuPPPPPPuu..", //
                "...uu.uu.uu...", //
                "...nn.nn.nn...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "stego_idle",
        outline: true,
        frames: &[
            &[
                "..............", //
                "..............", //
                ".....h........", //
                "...h.hH.h.....", //
                "..hH.hHhhH....", //
                "..PPPPPPPPP...", //
                ".PPPPPPPPPPPP.", //
                "PPuPPPPPPPPkPP", //
                "P.uPPPPPPPPPPP", //
                "..uPHHHHHHPu..", //
                "..uuPPPPPPuu..", //
                "..uu..uu.uu...", //
                "..nn..nn.nn...", //
                "..............", //
            ],
            &[
                "..............", //
                "..............", //
                "..............", //
                ".....h........", //
                "...h.hH.h.....", //
                "..hH.hHhhH....", //
                "..PPPPPPPPP...", //
                ".PPPPPPPPPPPP.", //
                "PPuPPPPPPPPuPP", //
                "P.uPPPPPPPPPPP", //
                "..uPHHHHHHPu..", //
                "..uu..uu.uu...", //
                "..nn..nn.nn...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "spino",
        outline: true,
        frames: &[
            &[
                "....a.a.......", //
                "...aoaoa......", //
                "..aoaoaoa.....", //
                "..aoaoaoa.NNN.", //
                "..NbbbbbbNbbbN", //
                ".NbbbbbbbbbwkN", //
                "NbbbBBBbbbbbbb", //
                "Nb.bBBBbbbIkIk", //
                "N...bBBbbN....", //
                "....NbBbN.....", //
                "....NNbbN.....", //
                "....NN..bb....", //
                "...NNN..bbb...", //
                "..............", //
            ],
            &[
                "....a.a.......", //
                "...aoaoa......", //
                "..aoaoaoa.....", //
                "..aoaoaoa.NNN.", //
                "..NbbbbbbNbbbN", //
                ".NbbbbbbbbbwkN", //
                "NbbbBBBbbbbbbb", //
                "Nb.bBBBbbbIkIk", //
                "N...bBBbbN....", //
                "....NbBbN.....", //
                "....NNbbN.....", //
                ".....Nbbb.....", //
                ".....NNbbb....", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "spino_idle",
        outline: true,
        frames: &[
            &[
                "....a.a.......", //
                "...aoaoa......", //
                "..aoaoaoa.....", //
                "..aoaoaoa.NNN.", //
                "..NbbbbbbNbbbN", //
                ".NbbbbbbbbbwkN", //
                "NbbbBBBbbbbbbb", //
                "Nb.bBBBbbbIkIk", //
                "N...bBBbbN....", //
                "....NbBbN.....", //
                "....NNbbN.....", //
                "....NN..bb....", //
                "...NNN..bbb...", //
                "..............", //
            ],
            &[
                "..............", //
                "....a.a.......", //
                "...aoaoa......", //
                "..aoaoaoa.....", //
                "..aoaoaoa.NNN.", //
                "..NbbbbbbNbbbN", //
                ".NbbbbbbbbbbbN", //
                "NbbbBBBbbbbbbb", //
                "Nb.bBBBbbbIkIk", //
                "N...bBBbbN....", //
                "....NbBbN.....", //
                "....NN..bb....", //
                "...NNN..bbb...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "pachy",
        outline: true,
        frames: &[
            &[
                "..............", //
                ".......wwww...", //
                "......wpwwww..", //
                "......wwppwpw.", //
                ".....ZxxxxxxZ.", //
                ".....xxxxwkxx.", //
                ".....xxxxxxxY.", //
                "...xZZxxxZZ...", //
                ".xxZZZxxxZx...", //
                "xx..ZZZxxY....", //
                "....ZZZZZ.....", //
                "....zz..ZZ....", //
                "...zzz..ZZZ...", //
                "..............", //
            ],
            &[
                "..............", //
                ".......wwww...", //
                "......wpwwww..", //
                "......wwppwpw.", //
                ".....ZxxxxxxZ.", //
                ".....xxxxwkxx.", //
                ".....xxxxxxxY.", //
                "...xZZxxxZZ...", //
                ".xxZZZxxxZx...", //
                "xx..ZZZxxY....", //
                "....ZZZZZ.....", //
                ".....zZZZ.....", //
                ".....zzZZZ....", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "pachy_idle",
        outline: true,
        frames: &[
            &[
                "..............", //
                ".......wwww...", //
                "......wpwwww..", //
                "......wwppwpw.", //
                ".....ZxxxxxxZ.", //
                ".....xxxxwkxx.", //
                ".....xxxxxxxY.", //
                "...xZZxxxZZ...", //
                ".xxZZZxxxZx...", //
                "xx..ZZZxxY....", //
                "....ZZZZZ.....", //
                "....zz..ZZ....", //
                "...zzz..ZZZ...", //
                "..............", //
            ],
            &[
                "..............", //
                "..............", //
                ".......wwww...", //
                "......wpwwww..", //
                "......wwppwpw.", //
                ".....ZxxxxxxZ.", //
                ".....xxxxZZxx.", //
                ".....xxxxxxxY.", //
                "...xZZxxxZZ...", //
                ".xxZZZxxxZx...", //
                "xx..ZZZxxY....", //
                "....zz..ZZ....", //
                "...zzz..ZZZ...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "bolt",
        outline: true,
        frames: &[
            &[
                ".ay.", //
                "aYYy", //
                "oyYa", //
                ".oa.", //
            ],
            &[
                ".ya.", //
                "yYYa", //
                "aYyo", //
                ".ao.", //
            ],
        ],
    },
    SpriteDef {
        name: "horn",
        outline: true,
        frames: &[
            &[
                ".ww.", //
                "wYYp", //
                ".wp.", //
            ],
            &[
                ".wp.", //
                "wYYp", //
                ".ww.", //
            ],
        ],
    },
    SpriteDef {
        name: "feather_shot",
        outline: true,
        frames: &[
            &[
                "...i", //
                ".iIi", //
                "iIb.", //
                "b...", //
            ],
            &[
                "...I", //
                ".iIi", //
                "iib.", //
                "b...", //
            ],
        ],
    },
    SpriteDef {
        name: "claw",
        outline: true,
        frames: &[
            &[
                "rr..", //
                ".wr.", //
                "..wr", //
                "..wr", //
                ".wr.", //
                "rr..", //
            ],
            &[
                "cr..", //
                ".rw.", //
                "..rw", //
                "..rw", //
                ".rw.", //
                "cr..", //
            ],
        ],
    },
    SpriteDef {
        name: "spike",
        outline: true,
        frames: &[
            &[
                ".h.", //
                "hHh", //
                "hHh", //
                ".u.", //
            ],
            &[
                ".H.", //
                "hHh", //
                "hhh", //
                ".u.", //
            ],
        ],
    },
    SpriteDef {
        name: "bubble",
        outline: true,
        frames: &[
            &[
                ".ii.", //
                "iI.i", //
                "i..i", //
                ".ii.", //
            ],
            &[
                ".ii.", //
                "i.Ii", //
                "i..i", //
                ".ii.", //
            ],
        ],
    },
    SpriteDef {
        name: "pebble",
        outline: true,
        frames: &[
            &[
                ".pp.", //
                "pwpe", //
                "epee", //
                ".ee.", //
            ],
            &[
                ".pe.", //
                "pppe", //
                "pwee", //
                ".ee.", //
            ],
        ],
    },
    SpriteDef {
        name: "lock",
        outline: true,
        frames: &[&[
            ".vvvv.", //
            "v....v", //
            "v....v", //
            "yyyyyy", //
            "yaakaa", //
            "yaakaa", //
            "oaaaao", //
        ]],
    },
    SpriteDef {
        name: "medal",
        outline: true,
        frames: &[&[
            "b.....b", //
            ".b...b.", //
            "..rrr..", //
            ".yYYYy.", //
            "yYyyyay", //
            "yyyyyay", //
            ".yaaaa.", //
            "..aaa..", //
        ]],
    },
    SpriteDef {
        name: "medal_off",
        outline: true,
        frames: &[&[
            "s.....s", //
            ".s...s.", //
            "..sss..", //
            ".dvvvd.", //
            "dvddddd", //
            "ddddddd", //
            ".ddddd.", //
            "..ddd..", //
        ]],
    },
    SpriteDef {
        name: "up_hp",
        outline: true,
        frames: &[&[
            "..gggggg..", //
            ".glllllLg.", //
            "glLlllllfg", //
            "glllgglllg", //
            "gllgffgllg", //
            "gllgffgllg", //
            ".gllggllg.", //
            "..glllfg..", //
            "...gffg...", //
            "....gg....", //
        ]],
    },
    SpriteDef {
        name: "up_damage",
        outline: true,
        frames: &[&[
            "..wwwwww..", //
            ".wwwwwwwp.", //
            "wwwwwwwwwp", //
            "wwwwwwwwwp", //
            ".wwwwwwpp.", //
            ".wwwpwwp..", //
            "..wwpwwp..", //
            "..wwppp...", //
            "...wp.....", //
            "...p......", //
        ]],
    },
    SpriteDef {
        name: "up_speed",
        outline: true,
        frames: &[&[
            "....ZZ....", //
            "...ZxxZ...", //
            "ZZ.ZxxZ.ZZ", //
            "ZxZ.ZZ.ZxZ", //
            ".Z.ZZZZ.Z.", //
            "..ZxxxxZ..", //
            ".ZxxxxxxZ.", //
            ".ZxxxxxxZ.", //
            "..ZxxxxZ..", //
            "...ZZZZ...", //
        ]],
    },
    SpriteDef {
        name: "up_rate",
        outline: true,
        frames: &[&[
            "w.....w...", //
            "pw.....w..", //
            ".pw.....w.", //
            "w.pw.....w", //
            "pw.pw....p", //
            ".pw.pw..p.", //
            "..pw.pw...", //
            "...p..p...", //
            "..........", //
            "..........", //
        ]],
    },
    SpriteDef {
        name: "up_crit",
        outline: true,
        frames: &[&[
            "...yyyy...", //
            ".yyaaaayy.", //
            "yaaawwaaay", //
            "yaawkkwaay", //
            "yaawkkwaay", //
            "yaaawwaaay", //
            ".yyaaaayy.", //
            "...yyyy...", //
        ]],
    },
    SpriteDef {
        name: "up_pickup",
        outline: true,
        frames: &[&[
            "..i....i..", //
            "...i..i...", //
            "i.......i.", //
            ".i..II....", //
            "...IiiB..i", //
            "..IiiiiB..", //
            "..iiiiBB..", //
            "...iBBb...", //
            "....bb....", //
        ]],
    },
    SpriteDef {
        name: "up_xp",
        outline: true,
        frames: &[&[
            "....Y.....", //
            "....y.....", //
            "...yYy....", //
            "yyyyYyyyy.", //
            ".yyYYYyy..", //
            "..ayyyya..", //
            "..ay.aya..", //
            ".aa...aa..", //
            ".a.....a..", //
        ]],
    },
    SpriteDef {
        name: "up_dash",
        outline: true,
        frames: &[&[
            "......ii..", //
            "iiii...Ii.", //
            "......IIIi", //
            "..iiiIIIIi", //
            "......IIIi", //
            "iiii...Ii.", //
            "......ii..", //
        ]],
    },
    SpriteDef {
        name: "up_bones",
        outline: true,
        frames: &[&[
            ".ww.......", //
            "wwww......", //
            "wwwwp.....", //
            ".wpwwp....", //
            "...pwwp...", //
            "....pwwp..", //
            ".....pwwpw", //
            "......pwww", //
            "......wwww", //
            ".......ww.", //
        ]],
    },
    SpriteDef {
        name: "tab_dot",
        outline: true,
        frames: &[&[
            "yy", //
            "yy", //
        ]],
    },
];
