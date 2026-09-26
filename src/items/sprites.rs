//! Item icons (14x14 art, 16x16 with outline), item projectiles, and the
//! color swatches beams and auras take their tint from.

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
            "..............", //
            "......r.......", //
            ".....ro.......", //
            ".....roo..r...", //
            "....roao..ro..", //
            "...roaaorroo..", //
            "...roayaooao..", //
            "..roaayyaaaor.", //
            "..roayYYyaaor.", //
            "..roayYYYyaor.", //
            "...roayYyaor..", //
            "....rooaaor...", //
            ".....rrrrr....", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "twin_barrel",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            ".sssssssss....", //
            ".peeeeeeeesk..", //
            ".evvvvvvvvsk..", //
            ".sssssssss....", //
            "...zZz........", //
            ".sssssssss....", //
            ".peeeeeeeesk..", //
            ".evvvvvvvvsk..", //
            ".sssssssss....", //
            "...zZz........", //
            "...zzz........", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "rubber_ball",
        outline: true,
        frames: &[&[
            "..............", //
            ".....hhhh.....", //
            "...hhHHhhhh...", //
            "..hHHwHhhhhP..", //
            "..hHwHhhhhhP..", //
            ".hhHHhhhhhhPP.", //
            ".hhhhhhhhhPPP.", //
            ".hhhhhhhhhPPP.", //
            ".hhhhhhhhPPPu.", //
            "..hhhhhhPPPu..", //
            "..PPhhPPPPuu..", //
            "...PPPPPPuu...", //
            ".....uuuu.....", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "drill_bit",
        outline: true,
        frames: &[&[
            "..............", //
            "...........pw.", //
            "..........pwe.", //
            ".........pwe..", //
            "........pwve..", //
            ".......ewve...", //
            "......pwve....", //
            ".....ewve.....", //
            "....zzve......", //
            "...zZZz.......", //
            "..zZZz........", //
            ".zZZz.........", //
            ".zzz..........", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "powder_keg",
        outline: true,
        frames: &[&[
            "..............", //
            "........y.Y...", //
            ".........a....", //
            "........k.....", //
            "....zzzzkzz...", //
            "...zZZZZZZZz..", //
            "...sssssssss..", //
            "...zZZxZZZZz..", //
            "...zZxZZZZZz..", //
            "...sssssssss..", //
            "...zZZZZZZZz..", //
            "....zzzzzzz...", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "spark_plug",
        outline: true,
        frames: &[&[
            "..............", //
            "......ss......", //
            ".....seps.....", //
            ".....sees.....", //
            "....wwwwww....", //
            "....wppppw....", //
            "....wppppw....", //
            "....wwwwww....", //
            ".....svvs.....", //
            ".....svvs.....", //
            "......ss......", //
            "......i.......", //
            ".....iI.......", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "black_coffee",
        outline: true,
        frames: &[&[
            "..............", //
            "....p..p......", //
            ".....p..p.....", //
            "....p..p......", //
            "..............", //
            "..wwwwwww.....", //
            "..wzzzzzwww...", //
            "..wzkkkzw.w...", //
            "..wzkkkzw.w...", //
            "..wzzzzzwww...", //
            "..wwwwwww.....", //
            "...pppppp.....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "magnet",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "....rrrrrr....", //
            "...rrHHHrrr...", //
            "..rrH....rrr..", //
            "..rr......rr..", //
            "..rr......rr..", //
            "..rr......rr..", //
            "..pp......pp..", //
            "..ww......ww..", //
            "..pp......pp..", //
            "..............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "heart_jar",
        outline: true,
        frames: &[&[
            "..............", //
            "....ssssss....", //
            "....eeeeee....", //
            "...pI....Ip...", //
            "..p.I....I.p..", //
            "..p..rr.rr.p..", //
            "..p.rHrrrrcp..", //
            "..p.rrrrrrcp..", //
            "..p..rrrrc.p..", //
            "..p...rrc..p..", //
            "..p....c...p..", //
            "...pppppppp...", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "static_coil",
        outline: true,
        frames: &[&[
            "..............", //
            ".....bbbb.....", //
            "....bBBBBb....", //
            ".....bbbb.....", //
            "....bBBBBb....", //
            ".....bbbb.....", //
            "....bBBBBb....", //
            ".....bbbb.....", //
            "....bBBBBb....", //
            ".....bbbb.....", //
            "......iI......", //
            ".....iIi......", //
            "......i.......", //
            "..............", //
        ]],
    },
    // Weapon icons.
    SpriteDef {
        name: "scattergun",
        outline: true,
        frames: &[&[
            "..............", //
            "...........y..", //
            "........y.....", //
            "..ssssss...a.y", //
            ".seeeeeeps....", //
            "sppppppppwy.a.", //
            "svvvvvvvvs..y.", //
            ".sssssZss.....", //
            "....ZZz...y...", //
            "...ZZz........", //
            "..ZZz.........", //
            ".zZz..........", //
            ".zz...........", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "railgun",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "...........I..", //
            "..........iIi.", //
            "sNNNNNNNNNiIIi", //
            "sBbBbBbBbBbiI.", //
            "sbBbBbBbBbBiI.", //
            "sNNNNNNNNNiIIi", //
            ".svvs.....iIi.", //
            ".svs.......I..", //
            "svs...........", //
            "ss............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "saw_ring",
        outline: true,
        frames: &[&[
            "......w.......", //
            "..w..wpw..w...", //
            "...wwpppww....", //
            "..wppeeeppw...", //
            "wwpeesssepww..", //
            ".wpesdddsepw..", //
            ".wpesdkdsepw..", //
            ".wpesdddsepw..", //
            "wwpeesssepww..", //
            "..wppeeeppw...", //
            "...wwpppww....", //
            "..w..wpw..w...", //
            "......w.......", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "bone_rang",
        outline: true,
        frames: &[&[
            "..............", //
            "..www.........", //
            ".wxxww........", //
            ".wxxxxw.......", //
            "..wZxxxw......", //
            "....wZxxw.....", //
            "......wZxw....", //
            ".......wZxw...", //
            "......wZxw....", //
            "....wZxxw.....", //
            "..wZxxxw......", //
            ".wxxxxw.......", //
            ".wxxww........", //
            "..www.........", //
        ]],
    },
    SpriteDef {
        name: "tesla_rod",
        outline: true,
        frames: &[&[
            "...i......i...", //
            "....i.II.i....", //
            ".....IiiI.....", //
            "....IiBBiI....", //
            "....iBbbBi....", //
            ".....IiiI.....", //
            "......ss......", //
            ".....sees.....", //
            "......ss......", //
            ".....sees.....", //
            "......ss......", //
            ".....sees.....", //
            "....svvvvs....", //
            "...ssssssss...", //
        ]],
    },
    SpriteDef {
        name: "seeker_pod",
        outline: true,
        frames: &[&[
            "...........pw.", //
            "..........pww.", //
            ".........rrp..", //
            "........rHrr..", //
            ".......rHrr...", //
            "......rrrc....", //
            "..s..rrrc.....", //
            "..ssrrrc......", //
            "...srrc.......", //
            "...ssc.ss.....", //
            "..oa..ss......", //
            ".oya..........", //
            "yyo...........", //
            "ao............", //
        ]],
    },
    SpriteDef {
        name: "laser_eye",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "....ppppp.....", //
            "..ppwwwwwpp...", //
            ".pwwwrrrwwwp..", //
            "pwwwrhHhrwwwp.", //
            "pwwrhkkkhrwwpr", //
            "pwwrhkkkhrwwrH", //
            "pwwwrhhhrwwwpr", //
            ".pwwwrrrwwwp..", //
            "..ppwwwwwpp...", //
            "....ppppp.....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "mine_layer",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "......s.......", //
            "......r.......", //
            ".....rHr......", //
            "...sssrsss....", //
            "..svvvvvvvs...", //
            ".svveeeevvvs..", //
            "svveppppevvvs.", //
            "svvvvvvvvvvvs.", //
            "sssssssssssss.", //
            "kdkdkdkdkdkdk.", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "magma_core",
        outline: true,
        frames: &[&[
            "..............", //
            ".....dddd.....", //
            "...ddsssdd....", //
            "..dsoasssyd...", //
            ".dssyaosoasd..", //
            ".dsssoYasssd..", //
            ".dsaoYYoasdd..", //
            ".dssssoasssd..", //
            ".dsyassoysd...", //
            "..dsoasssod...", //
            "...ddsossd....", //
            ".....dddd.....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "frost_halo",
        outline: true,
        frames: &[&[
            "......I.......", //
            "...I..i..I....", //
            "....I.i.I.....", //
            ".....IBI......", //
            ".I...BiB...I..", //
            "..I.BiIiB.I...", //
            "IiiiiIBIiiiiI.", //
            "..I.BiIiB.I...", //
            ".I...BiB...I..", //
            ".....IBI......", //
            "....I.i.I.....", //
            "...I..i..I....", //
            "......I.......", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "meteor_call",
        outline: true,
        frames: &[&[
            "..............", //
            "Y.....y.......", //
            ".ya....a......", //
            "..yao...o.....", //
            "...aaor.......", //
            "....aorzzz....", //
            ".....ozZZZz...", //
            "....ozZxxZZz..", //
            "....zZxxZZzz..", //
            "....zZZZZzzz..", //
            ".....zZZzzz...", //
            "......zzzz....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "quill_burst",
        outline: true,
        frames: &[&[
            "..............", //
            "......w.......", //
            "..w...x...w...", //
            "...x..x..x....", //
            "....x.x.x.....", //
            ".....zZz......", //
            "wxxx.ZxZ.xxxw.", //
            ".....zZz......", //
            "....x.x.x.....", //
            "...x..x..x....", //
            "..w...x...w...", //
            "......w.......", //
            "..............", //
            "..............", //
        ]],
    },
    // Active icons.
    SpriteDef {
        name: "starfall",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "i.......y.....", //
            ".iB....yYy....", //
            "..iBi..yYy....", //
            "...BiyyyYyyyy.", //
            "....iyYYYYYy..", //
            ".....yYYYYy...", //
            "......yYYYy...", //
            ".....yYy.yYy..", //
            ".....yy...yy..", //
            "..............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "cold_snap",
        outline: true,
        frames: &[&[
            "..............", //
            "......I.......", //
            ".....IiI......", //
            "..I..IiB..I...", //
            ".IiI.IiB.IiI..", //
            ".IiB.IBB.IiB..", //
            ".IBB.iBN.iBN..", //
            ".iBN.BBN.BBN..", //
            "..BN.BNN.BN...", //
            "..NNNBBNNNN...", //
            "...NNNNNNN....", //
            "..............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "turtle_shell",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "....tttttt....", //
            "...tgglggft...", //
            "..tglLlfgLlt..", //
            ".tglLLlfglLlt.", //
            ".tgllfffgllft.", //
            ".tffftggtffft.", //
            ".tgflgllfgflt.", //
            "tttttttttttttt", //
            "xxZxxZxxZxxZxx", //
            ".ZZ........ZZ.", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "adrenaline",
        outline: true,
        frames: &[&[
            "...........ss.", //
            "..........sws.", //
            ".........svs..", //
            "........pIpp..", //
            ".......pIIIp..", //
            "......phhIp...", //
            ".....phhhp....", //
            "....phhhp.....", //
            "...phhhp......", //
            "..sppp........", //
            ".s.ss.........", //
            "w.............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "big_roar",
        outline: true,
        frames: &[&[
            "..............", //
            ".gggggg.......", //
            "glllwkg....i..", //
            "gllllgg..i..i.", //
            "gllggggw..i.i.", //
            "gllg..w.i..i.i", //
            "gllg.......i.i", //
            "gllg..w.i..i.i", //
            "glllgggw..i.i.", //
            "glllllgg.i..i.", //
            ".gggggg....i..", //
            "..............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "bone_storm",
        outline: true,
        frames: &[&[
            ".ww.......ww..", //
            "wpw.......wpw.", //
            ".wpw.....wpw..", //
            "..wpw...wpw...", //
            "...wpw.wpw....", //
            "....wpwpw.....", //
            ".....wpw......", //
            "....wpwpw.....", //
            "...wpw.wpw....", //
            "..wpw...wpw...", //
            ".wpw.....wpw..", //
            "wpw.......wpw.", //
            ".ww.......ww..", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "med_kit",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            ".....ssss.....", //
            "....s....s....", //
            ".wwwwwwwwwwww.", //
            ".wppppccppppw.", //
            ".wppppcrppppw.", //
            ".wppcccrrrppw.", //
            ".wpprrrrrrppw.", //
            ".wppppcrppppw.", //
            ".wppppcrppppw.", //
            ".eeeeeeeeeeee.", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "gravity_egg",
        outline: true,
        frames: &[&[
            "..............", //
            "......uu......", //
            "....uuPPuu....", //
            "...uPPhPPPu...", //
            "...uPhPuuPu...", //
            "..uPPPuPPuPu..", //
            "..uPPuPPPuPu..", //
            "..uPPuPuuPPu..", //
            "..uPPPuPPPPu..", //
            "...uPPPPPPu...", //
            "...uuPPPPuu...", //
            ".....uuuu.....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "mine_burst",
        outline: true,
        frames: &[&[
            "..............", //
            "......r.......", //
            ".....sss......", //
            "....svvvs.....", //
            "....sssss.....", //
            "..............", //
            "..r.......r...", //
            ".sss.....sss..", //
            "svvvs...svvvs.", //
            "sssss...sssss.", //
            "..............", //
            "..............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "war_drum",
        outline: true,
        frames: &[&[
            "..x........x..", //
            "...x......x...", //
            "....Z....Z....", //
            ".....Z..Z.....", //
            "..wwwwwwwwww..", //
            ".wppppppppppw.", //
            ".cwwwwwwwwwwc.", //
            ".rryrrrryrrrc.", //
            ".rrryrryrrrrc.", //
            ".rrrryyrrrrrc.", //
            ".cwwwwwwwwwwc.", //
            "..cccccccccc..", //
            "..............", //
            "..............", //
        ]],
    },
    // Passive icons.
    SpriteDef {
        name: "iron_jaw",
        outline: true,
        frames: &[&[
            "..............", //
            ".sssssssssss..", //
            "speeeeeeeeeps.", //
            "sevvvvvvvvves.", //
            ".wvwvwvwvwvw..", //
            ".w.w.w.w.w.w..", //
            "..............", //
            ".w.w.w.w.w.w..", //
            ".wvwvwvwvwvw..", //
            "sevvvvvvvvves.", //
            "speeeeeeeeeps.", //
            ".sssssssssss..", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "sharp_tooth",
        outline: true,
        frames: &[&[
            "..............", //
            "...wwwwwww....", //
            "..wYYYYYYYw...", //
            "..wYYwwwYYw...", //
            "...wYYYYYw....", //
            "...wYYYYw.....", //
            "....wYYYw.....", //
            "....wYYw......", //
            ".....wYw......", //
            ".....wYw......", //
            ".....ww.......", //
            "......r.......", //
            "......c.......", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "raptor_legs",
        outline: true,
        frames: &[&[
            "..............", //
            "........gg....", //
            "........gfg...", //
            ".......ggfg...", //
            "..pp...gfg....", //
            ".......gfg....", //
            "pppp..ggfg....", //
            "......gfg.....", //
            "..pp.ggfg.....", //
            "....gggg......", //
            "...gwgggw.....", //
            "..gw.gw.gw....", //
            "..w..w...w....", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "big_bones",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "..............", //
            ".ww........ww.", //
            "wppw......wppw", //
            "wpppwwwwwwpppw", //
            ".weppppppppew.", //
            ".weeeeeeeeeew.", //
            "weeewwwwwweeew", //
            "weew......weew", //
            ".ww........ww.", //
            "..............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "amber_resin",
        outline: true,
        frames: &[&[
            "......aa......", //
            ".....aYya.....", //
            "....aYyyya....", //
            "...aYyyyyao...", //
            "..aYyykyyyao..", //
            "..ayykkkyyao..", //
            ".ayyykkkyyyao.", //
            ".ayyyykyyyyao.", //
            ".ayyyyyyyyaoo.", //
            "..aoyyyyyaoo..", //
            "..aooaaaoooo..", //
            "...oooooooo...", //
            "....oooooo....", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "feather",
        outline: true,
        frames: &[&[
            "...........Bi.", //
            "..........BiI.", //
            ".........BiIB.", //
            "........BiIBb.", //
            ".......BiIBb..", //
            "......BiIBb...", //
            ".....BiIBb....", //
            "....BiIBb.....", //
            "...BiIBb......", //
            "...iIBb.......", //
            "..sBb.........", //
            ".s............", //
            "s.............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "moss_pouch",
        outline: true,
        frames: &[&[
            "......gl......", //
            ".....g.lg.....", //
            ".....zzzz.....", //
            "......zz......", //
            "....ffffff....", //
            "...ffggggff...", //
            "..ffgglggggf..", //
            "..fggLlggggf..", //
            "..fgggggglgf..", //
            "..fggggggggf..", //
            "...ffggggff...", //
            "....ffffff....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "tail_club",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "g.............", //
            "gg............", //
            ".gg...........", //
            "..ggg.........", //
            "...gggg.......", //
            ".....ggxxx....", //
            "......xZZZx...", //
            ".....xZwxZZx..", //
            ".....xZZZZZx..", //
            "......xZZZx...", //
            ".......xxx....", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "frost_fang",
        outline: true,
        frames: &[&[
            "..............", //
            "..IIIIIIII..I.", //
            "..IiiiiiiI.IiI", //
            "..IiiBiiiI..I.", //
            "...IiiiBI.....", //
            "...IiiBiI.....", //
            "....IiBI......", //
            "....IiBI......", //
            ".....IB..I....", //
            ".....IB.IiI...", //
            "......B..I....", //
            "..............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "blood_drop",
        outline: true,
        frames: &[&[
            "......r.......", //
            "......r.......", //
            ".....rrr......", //
            ".....rHr......", //
            "....rrHrr.....", //
            "....rHrrr.....", //
            "...rrrrrrc....", //
            "...rrrrrrc....", //
            "..rrrrrrrcc...", //
            "..crrrrrccc...", //
            "...ccccccc....", //
            "....mmmmm.....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "target_lock",
        outline: true,
        frames: &[&[
            "......r.......", //
            "......r.......", //
            "....rrrrr.....", //
            "...r..r..r....", //
            "..r...r...r...", //
            "..r.......r...", //
            "rrrrr.h.rrrrr.", //
            "..r.......r...", //
            "..r...r...r...", //
            "...r..r..r....", //
            "....rrrrr.....", //
            "......r.......", //
            "......r.......", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "long_neck",
        outline: true,
        frames: &[&[
            "..........ggg.", //
            ".........glkg.", //
            ".........gggg.", //
            ".........gl...", //
            "........gl....", //
            "........gl....", //
            ".......gl.....", //
            ".......gl.....", //
            "......gll.....", //
            ".....glll.....", //
            "..gggglllg....", //
            ".glllllllgg...", //
            "glllllllllg...", //
            ".gf.gf..gf....", //
        ]],
    },
    SpriteDef {
        name: "boulder_shot",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "....zzzzz.....", //
            "...zZZxZZz....", //
            "..zZxxZZZZz...", //
            ".zZxxZZZZZZz..", //
            ".zZxZZZZZzZz..", //
            ".zZZZZZzZZZz..", //
            ".zZZZZZZZZzz..", //
            ".zzZZzZZZzzz..", //
            "..zzZZZZzzz...", //
            "...zzzzzzz....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "lucky_claw",
        outline: true,
        frames: &[&[
            "...........y..", //
            "..........yYy.", //
            "...........y..", //
            "..w...........", //
            ".wpw..w.......", //
            ".wpw.wpw..w...", //
            "..wpwwpw.wpw..", //
            "..wpwwpwwpw...", //
            "...zZZZZzpw...", //
            "...zZZZZZzw...", //
            "...zZZZZZz....", //
            "....zZZZz.....", //
            ".....zzz......", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "thorn_hide",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "...w...w...w..", //
            "...p...p...p..", //
            "..wpw.wpw.wpw.", //
            "..ggg.ggg.ggg.", //
            ".ggggggggggggg", //
            ".glllllllllllg", //
            ".gllfllllfllgg", //
            ".glllllfllllg.", //
            ".ggggggggggg..", //
            "..............", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "shrapnel",
        outline: true,
        frames: &[&[
            "..............", //
            ".s.....e......", //
            "..s...ev...s..", //
            "......v...s...", //
            "...ssvvvs.....", //
            "..s.veeevs....", //
            "...vepppev..e.", //
            "...vepYpev.e..", //
            "...veppev.....", //
            "....vvvv..s...", //
            "..e....s......", //
            ".e......s.....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "spore_pod",
        outline: true,
        frames: &[&[
            "..............", //
            "..l.......l...", //
            "......L.......", //
            "...hhhhhhh....", //
            "..hHhhhHhhh...", //
            ".hHHhhhhhHhh..", //
            ".hhhhHhhhhhh..", //
            "..uuuuuuuuu...", //
            ".....pwp......", //
            ".....pwp......", //
            ".....pwp......", //
            "....ppwpp.....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "glass_cannon",
        outline: true,
        frames: &[&[
            "..............", //
            "..............", //
            "..........IIi.", //
            ".........IiiI.", //
            "........IiBiI.", //
            ".......IiBiI..", //
            "......IiBiI...", //
            ".....IkBiI....", //
            "....IiBkI.....", //
            "...IiBiIk.....", //
            "..zZZIiI......", //
            ".zZxxZz.......", //
            ".zZxZz........", //
            "..zzz.........", //
        ]],
    },
    SpriteDef {
        name: "storm_cell",
        outline: true,
        frames: &[&[
            "..............", //
            "....ssss......", //
            "..sseeeess....", //
            ".sepppppees...", //
            "seppppppppes..", //
            "sseeeeeeeess..", //
            ".sssssyssss...", //
            ".....yy.......", //
            "....yY........", //
            "...yYYy.......", //
            ".....Yy.......", //
            "....yY........", //
            "....y.........", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "fossil",
        outline: true,
        frames: &[&[
            "..............", //
            "....zzzzzz....", //
            "...zxxxxxxz...", //
            "..zxZZZZZxxz..", //
            ".zxZzzzzZZxz..", //
            ".zxZzxxzZxxz..", //
            ".zxZzxZzZxxz..", //
            ".zxZzzZzZxxz..", //
            ".zxZZZZzZxz...", //
            "..zxzzzzZxz...", //
            "..zxxxxxxz....", //
            "...zzzzzz.....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "golden_egg",
        outline: true,
        frames: &[&[
            "..............", //
            "......yy......", //
            "....yyYYyy....", //
            "...yYYYyyya...", //
            "...yYYyyyya...", //
            "..yYYyyyyyaa..", //
            "..yYyyyyyyaa..", //
            "..yyyyyyyaao..", //
            "..ayyyyyaaao..", //
            "...aayyaaao...", //
            "...oaaaaaoo...", //
            ".....oooo.....", //
            "..............", //
            "..............", //
        ]],
    },
    SpriteDef {
        name: "hot_foot",
        outline: true,
        frames: &[&[
            "....y.........", //
            "...yao..y.....", //
            "...aro.yao....", //
            "..orr..aro....", //
            "..rrr..orr.a..", //
            "..rrr..rrr.ao.", //
            "...r...rrr.or.", //
            "..rrrrrrrrrrr.", //
            ".rrrrrrrrrrrr.", //
            ".rrrrrrrrrrr..", //
            "..rrrrrrrrr...", //
            "...ccccccc....", //
            "..............", //
            "..............", //
        ]],
    },
    // Projectiles, blades, mines, meteors.
    SpriteDef {
        name: "pellet",
        outline: true,
        frames: &[&[
            "Ya", //
            "ao", //
        ]],
    },
    SpriteDef {
        name: "slug",
        outline: true,
        frames: &[
            &[
                ".Ii.", //
                "IIIi", //
                "iIIb", //
                ".ib.", //
            ],
            &[
                ".iI.", //
                "iIII", //
                "bIIi", //
                ".bi.", //
            ],
        ],
    },
    SpriteDef {
        name: "drill_slug",
        outline: true,
        frames: &[
            &[
                "..ay..", //
                ".aYya.", //
                "ayYaoy", //
                "yoaYya", //
                ".ayYa.", //
                "..ya..", //
            ],
            &[
                "..ya..", //
                ".ayYa.", //
                "yoaYya", //
                "ayYaoy", //
                ".aYya.", //
                "..ay..", //
            ],
        ],
    },
    SpriteDef {
        name: "saw",
        outline: true,
        frames: &[
            &[
                "...p...", //
                ".pwwwe.", //
                ".wesew.", //
                "pwsdswp", //
                ".wesew.", //
                ".ewwwp.", //
                "...p...", //
            ],
            &[
                ".p...e.", //
                "..www..", //
                ".wesew.", //
                ".wsdsw.", //
                ".wesew.", //
                "..www..", //
                ".e...p.", //
            ],
        ],
    },
    SpriteDef {
        name: "fire_saw",
        outline: true,
        frames: &[
            &[
                "...a...", //
                ".aYYYo.", //
                ".Yoroy.", //
                "aYrmrYa", //
                ".Yoroy.", //
                ".oYYYa.", //
                "...a...", //
            ],
            &[
                ".a...o.", //
                "..YYY..", //
                ".Yoroy.", //
                ".Yrmry.", //
                ".Yoroy.", //
                "..YYY..", //
                ".o...a.", //
            ],
        ],
    },
    SpriteDef {
        name: "rang",
        outline: true,
        frames: &[
            &[
                "..xw..", //
                ".xZ...", //
                "xZ....", //
                "xZ....", //
                ".xZ...", //
                "..xw..", //
            ],
            &[
                "......", //
                "..xx..", //
                ".xZZx.", //
                "xZ..Zx", //
                "w....w", //
                "......", //
            ],
        ],
    },
    SpriteDef {
        name: "plasma_rang",
        outline: true,
        frames: &[
            &[
                "..iI..", //
                ".iB...", //
                "iBI...", //
                "iBI...", //
                ".iB...", //
                "..iI..", //
            ],
            &[
                "......", //
                "..ii..", //
                ".iBBi.", //
                "iBIIBi", //
                "I....I", //
                "......", //
            ],
        ],
    },
    SpriteDef {
        name: "missile",
        outline: true,
        frames: &[&[
            ".wr.", //
            "wrHc", //
            "rrrc", //
            ".cc.", //
        ]],
    },
    SpriteDef {
        name: "mini_missile",
        outline: true,
        frames: &[&[
            ".r.", //
            "rHr", //
            ".c.", //
        ]],
    },
    SpriteDef {
        name: "mine",
        outline: true,
        frames: &[
            &[
                "..rr..", //
                ".svvs.", //
                "svvvvs", //
                "ssssss", //
                "kdkdkd", //
            ],
            &[
                "..cc..", //
                ".svvs.", //
                "svvvvs", //
                "ssssss", //
                "kdkdkd", //
            ],
        ],
    },
    SpriteDef {
        name: "cluster_mine",
        outline: true,
        frames: &[
            &[
                "...r...", //
                ".s.r.s.", //
                "sPvvvPs", //
                "svPvPvs", //
                "sssssss", //
                ".k.k.k.", //
            ],
            &[
                "...c...", //
                ".s.c.s.", //
                "sPvvvPs", //
                "svPvPvs", //
                "sssssss", //
                ".k.k.k.", //
            ],
        ],
    },
    SpriteDef {
        name: "meteor",
        outline: true,
        frames: &[
            &[
                "..zzz..", //
                ".zZxZz.", //
                "zZxZZZz", //
                "zZZZzZz", //
                "zZZzZZz", //
                ".zZZZz.", //
                "..zzz..", //
            ],
            &[
                "..zzz..", //
                ".zZZZz.", //
                "zZZZxZz", //
                "zZzZZZz", //
                "zZZZZzz", //
                ".zzZZz.", //
                "..zzz..", //
            ],
        ],
    },
    SpriteDef {
        name: "big_meteor",
        outline: true,
        frames: &[&[
            "...zzzzz...", //
            "..zZZxZZz..", //
            ".zZxxZZZZz.", //
            "zZxxZZzZZZz", //
            "zZxZZZZZZZz", //
            "zZZZZzZZZzz", //
            "zZZzZZZZZZz", //
            "zZZZZZZzZzz", //
            ".zZZZZZZzz.", //
            "..zzZZZzz..", //
            "...zzzzz...", //
        ]],
    },
    SpriteDef {
        name: "quill",
        outline: true,
        frames: &[&[
            ".w.", //
            "xZx", //
            ".x.", //
        ]],
    },
    SpriteDef {
        name: "big_quill",
        outline: true,
        frames: &[&[
            "..w..", //
            ".wxw.", //
            "wxZxw", //
            ".xZx.", //
            "..x..", //
        ]],
    },
    SpriteDef {
        name: "pinball",
        outline: true,
        frames: &[&[
            ".HH.", //
            "HwHh", //
            "hHhP", //
            ".PP.", //
        ]],
    },
    // Color swatches for beams and auras: the most used color is the body,
    // the second the highlight.
    SpriteDef { name: "laser", outline: false, frames: &[&["rrr", "rhr", "rrr"]] },
    SpriteDef { name: "ice_laser", outline: false, frames: &[&["iii", "iIi", "iii"]] },
    SpriteDef { name: "magma", outline: false, frames: &[&["ooo", "oao", "ooo"]] },
    SpriteDef { name: "frost", outline: false, frames: &[&["iii", "iBi", "iii"]] },
    SpriteDef { name: "tar", outline: false, frames: &[&["uuu", "uPu", "uuu"]] },
];
