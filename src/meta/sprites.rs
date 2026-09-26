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
        name: "rex_attack",
        outline: true,
        frames: &[
            &[
                ".......gggggg.", //
                "......glllllgg", //
                "......gllwklgg", //
                "......glllgggg", //
                "......gkwkwkk.", //
                "......goay....", //
                "...g..gkoa....", //
                "..gg.gffffwf..", //
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
                "...g..gko.....", //
                "..gg.gfffff...", //
                ".ggggggggxxgg.", //
                "gg..gfgggxx...", //
                "....ffgggg....", //
                "....ff..gg....", //
                "...fff..ggg...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "rex_dash",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "..............", //
            ".......ggggg..", //
            "......glllllg.", //
            "......gllwklgg", //
            "......glllgggg", //
            "ggg...gggggggg", //
            ".ggggggkwkwkk.", //
            "..gggggffffff.", //
            "....fgggxxgg..", //
            "...ff..gggg...", //
            "..ff.....gg...", //
            ".ff......ggg..", //
        ]],
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
        name: "trike_attack",
        outline: true,
        frames: &[
            &[
                "..............", //
                "..............", //
                "..............", //
                "..............", //
                ".........rr...", //
                "...aaa..rYrr..", //
                ".aaaaaarrrrr..", //
                "aooaaaaarrrwww", //
                ".oaaaaaaaaakaa", //
                "..oaaaaaaaaYY.", //
                "..oYYYYYYao...", //
                "..oo.oo..oo...", //
                "..zz.zz..zz...", //
                "..............", //
            ],
            &[
                "..............", //
                "..............", //
                "..............", //
                "........rr....", //
                ".......rYrr...", //
                "...aaa.rrrr..w", //
                ".aaaaaarrrraww", //
                "aooaaaaaarakaa", //
                ".oaaaaaaaaaaaw", //
                "..oaaaaaaaaYY.", //
                "..oYYYYYYao...", //
                "..oo.oo..oo...", //
                "..zz.zz..zz...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "trike_dash",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "..............", //
            "..............", //
            "..............", //
            ".........rr...", //
            "...aaa..rYrr..", //
            ".aaaaaarrrrr..", //
            "aooaaaaarrrwww", //
            ".oaaaaaaaaakaa", //
            "..oaaaaaaaaYY.", //
            ".ooYYYYYYao...", //
            "oo...oo...oo..", //
            "z.....zz...zz.", //
        ]],
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
        name: "ptera_attack",
        outline: true,
        frames: &[
            &[
                "......bb......", //
                ".....bBb......", //
                "..b.bBBb..oo..", //
                "..bbbBBb.oBBo.", //
                "...bbBBbbBBkYY", //
                ".....bBBBBB...", //
                "....bBBBBBbYY.", //
                "...bBBIIBBb...", //
                "......bIIb....", //
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
                ".bb......oBBo.", //
                "..bBbbbbbBBkYY", //
                "...bBBBBBBB...", //
                "....bBBBBBbYY.", //
                "..bbBBIIBBb...", //
                ".bb...bIIb....", //
                "......bbb.....", //
                "..............", //
                ".......y.y....", //
                "..............", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "ptera_dash",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "bb............", //
            ".bBbb.........", //
            "..bBBbb.......", //
            "...bBBBbb.oo..", //
            "....bBBBBoBBo.", //
            "...bbbBBBBBkYY", //
            "..bb.bBBIIBYY.", //
            "...yy..bIIb...", //
            "........bb....", //
            "..............", //
            "..............", //
            "..............", //
        ]],
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
        name: "raptor_attack",
        outline: true,
        frames: &[
            &[
                "..............", //
                "..............", //
                "..............", //
                "c.........rrr.", //
                "rc.......rrwkr", //
                ".rc......rrrrr", //
                "..rrc...rrYYY.", //
                "...rrcrrrr....", //
                "....rrrrcrrrYw", //
                ".....rYYYr...w", //
                "......rrc.....", //
                "......c..c....", //
                ".....cc...cc..", //
                ".....w.....w..", //
            ],
            &[
                "..............", //
                "..............", //
                "..............", //
                "..........rrr.", //
                "c........rrwkr", //
                ".rc......rrrrr", //
                "..rrc...rrYYY.", //
                "...rrcrrrr....", //
                "....rrrrcrrrY.", //
                ".....rYYYr..Y.", //
                "......rrc.....", //
                "......c..c....", //
                ".....cc...cc..", //
                ".....w.....w..", //
            ],
        ],
    },
    SpriteDef {
        name: "raptor_dash",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "..........rrr.", //
            ".........rrwkr", //
            ".........rrrrr", //
            "cc.......rYYY.", //
            ".rrcc...rrr...", //
            "...rrrcrrrrYw.", //
            ".....rYYYrc.w.", //
            "...cccc..cc...", //
            "..w.......cw..", //
            "..............", //
            "..............", //
            "..............", //
        ]],
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
        name: "stego_attack",
        outline: true,
        frames: &[
            &[
                ".....H........", //
                ".H...hH..H....", //
                "..h..hH.hH....", //
                "..hH.hH.hH....", //
                "H.hH.hHhhH....", //
                "hPPPPPPPPPP...", //
                "hPPPPPPPPPPPP.", //
                ".PuPPPPPPPPkPP", //
                "..uPPPPPPPPPPP", //
                "..uPHHHHHHPu..", //
                "..uuPPPPPPuu..", //
                "..uu.uu.uu.uu.", //
                "..nn.nn.nn.nn.", //
                "..............", //
            ],
            &[
                "..............", //
                ".....H........", //
                "..H..hH.H.....", //
                "..hH.hH.hH....", //
                "..hH.hHhhH....", //
                "H.PPPPPPPPP...", //
                "PPPPPPPPPPPPP.", //
                ".PuPPPPPPPPkPP", //
                "P.uPPPPPPPPPPP", //
                "..uPHHHHHHPu..", //
                "..uuPPPPPPuu..", //
                "..uu.uu.uu.uu.", //
                "..nn.nn.nn.nn.", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "stego_dash",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "H.............", //
            "hH.....h..h...", //
            ".P.h..hH.hH...", //
            ".PhH.hHhhH....", //
            "..PPPPPPPPP...", //
            ".PPPPPPPPPPPP.", //
            "..uPPPPPPPPPPP", //
            "..uPPPPPPPPkPP", //
            "..uPHHHHHHPPP.", //
            ".uu......uu...", //
            "nn........nn..", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "spino",
        outline: true,
        frames: &[
            &[
                "....y.........", //
                "...oaoy.......", //
                "..yoaoa..BB...", //
                "..aoaoaybwkbbB", //
                ".oaoaoaobbNNNb", //
                "..bbbbbbbbbbb.", //
                "..bbbbbbbbN...", //
                "..bbbbbbbBb...", //
                ".bbNbbbbBBbb..", //
                "bb..NBBBBb....", //
                "b...NNBBbb....", //
                "....NN..bb....", //
                "...NNN..bbb...", //
                "..............", //
            ],
            &[
                "....y.........", //
                "...oaoy.......", //
                "..yoaoa..BB...", //
                "..aoaoaybwkbbB", //
                ".oaoaoaobbNNNb", //
                "..bbbbbbbbbbb.", //
                "..bbbbbbbbN...", //
                "..bbbbbbbBb...", //
                ".bbNbbbbBBbb..", //
                "bb..NBBBBb....", //
                "b...NNBBbb....", //
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
                "....y.........", //
                "...oaoy.......", //
                "..yoaoa..BB...", //
                "..aoaoaybwkbbB", //
                ".oaoaoaobbNNNb", //
                "..bbbbbbbbbbb.", //
                "..bbbbbbbbN...", //
                "..bbbbbbbBb...", //
                ".bbNbbbbBBbb..", //
                "bb..NBBBBb....", //
                "b...NNBBbb....", //
                "....NN..bb....", //
                "...NNN..bbb...", //
                "..............", //
            ],
            &[
                "..............", //
                "....y.........", //
                "...oaoy.......", //
                "..yoaoa..BB...", //
                "..aoaoaybNNbbB", //
                ".oaoaoaobbNNNb", //
                "..bbbbbbbbbbb.", //
                "..bbbbbbbbN...", //
                "..bbbbbbbBb...", //
                ".bbNbbbbBBbb..", //
                "bb..NBBBBb....", //
                "b...NN..bb....", //
                "...NNN..bbb...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "spino_attack",
        outline: true,
        frames: &[
            &[
                "....y.........", //
                "...oaoy......B", //
                "..yoaoa..BBBBb", //
                "..aoaoaybwkbb.", //
                ".oaoaoaobbcc.i", //
                "..bbbbbbbbbc..", //
                "..bbbbbbbbbbbb", //
                "..bbbbbbbBb...", //
                ".bbNbbbbBBbb..", //
                "bb..NBBBBb....", //
                "b...NNBBbb....", //
                "....NN..bb....", //
                "...NNN..bbb...", //
                "..............", //
            ],
            &[
                "....y.........", //
                "...oaoy.......", //
                "..yoaoa..BBBBB", //
                "..aoaoaybwkbb.", //
                ".oaoaoaobbcc..", //
                "..bbbbbbbbbbbb", //
                "..bbbbbbbbN...", //
                "..bbbbbbbBb...", //
                ".bbNbbbbBBbb..", //
                "bb..NBBBBb....", //
                "b...NNBBbb....", //
                "....NN..bb....", //
                "...NNN..bbb...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "spino_dash",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "..............", //
            ".y..y.........", //
            ".oaooaoy......", //
            ".ooaooao.BB...", //
            "bbbbbbbbbwkbbB", //
            ".bbbbbbbbbNNNb", //
            "..Nbbbbbbbbbb.", //
            "...NNbbbBBbb..", //
            "..NNNbbBB.....", //
            "NNN...bbb.....", //
            ".......bbb....", //
            "..............", //
        ]],
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
        name: "pachy_attack",
        outline: true,
        frames: &[
            &[
                "..............", //
                "........wwww..", //
                ".......wpwwww.", //
                ".......wwppwpw", //
                "......ZxxxxxxZ", //
                "......xxxxwkxx", //
                "......xxxxxxxY", //
                "...xZZxxxxc..p", //
                ".xxZZZxxxZxxY.", //
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
                "...xZZxxxxc...", //
                ".xxZZZxxxZxY..", //
                "xx..ZZZxxY....", //
                "....ZZZZZ.....", //
                "....zz..ZZ....", //
                "...zzz..ZZZ...", //
                "..............", //
            ],
        ],
    },
    SpriteDef {
        name: "pachy_dash",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "..............", //
            "..............", //
            "..........www.", //
            ".........Zwpww", //
            "...xZZxxxxwwpw", //
            ".xxZZZxxwkZwpw", //
            "xx..ZZZxxxxZw.", //
            "....ZZZxxxY...", //
            "...ZZZZZZ.....", //
            "..zz.....ZZ...", //
            "zzz.......ZZZ.", //
            "..............", //
        ]],
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
