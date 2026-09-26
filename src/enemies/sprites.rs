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
        name: "beetle",
        outline: true,
        frames: &[
            &[
                "....ccccc....", //
                "..ccrrrrrcc..", //
                ".crYYrrrrrcnn", //
                ".crYrrmrrrcnw", //
                "crrrrrmrrrcnn", //
                "cmrrrrmrrrmc.", //
                ".cmmmmmmmmc..", //
                "..v.v..v.v...", //
                ".v...v..v....", //
            ],
            &[
                "....ccccc....", //
                "..ccrrrrrcc..", //
                ".crYYrrrrrcnn", //
                ".crYrrmrrrcnw", //
                "crrrrrmrrrcnn", //
                "cmrrrrmrrrmc.", //
                ".cmmmmmmmmc..", //
                "..v.v..v.v...", //
                "...v..v...v..", //
            ],
        ],
    },
    SpriteDef {
        name: "frog",
        outline: true,
        frames: &[
            &[
                "......ggg....", //
                ".....gwkwg...", //
                "..gggglwwgg..", //
                ".gllllllllgg.", //
                "gllLLllllllgk", //
                "glLLllllllmmk", //
                "gllllYYYYYYg.", //
                ".glYYYYYYYg..", //
                "..gYYYYYYg...", //
                ".ffg....gff..", //
                "fff......fff.", //
            ],
            &[
                "......ggg....", //
                ".....gwkwg...", //
                "..gggglwwgg..", //
                ".gllllllllgg.", //
                "gllLLllllllgk", //
                "glLLlllllllhk", //
                "gllllYYYYYYhh", //
                ".glYYYYYYYYg.", //
                "..gYYYYYYYg..", //
                ".ffg....gff..", //
                "fff......fff.", //
            ],
        ],
    },
    SpriteDef {
        name: "dilo",
        outline: true,
        frames: &[
            &[
                "..............", //
                "..........aa..", //
                ".........ayga.", //
                "........agggkg", //
                "........aggggY", //
                "..ff....agggr.", //
                ".fggfffffgga..", //
                "fgLLggggggg...", //
                "..fggLLLLgg...", //
                "...ffggggff...", //
                "...f.f..f.f...", //
                "..ff.f.ff.f...", //
            ],
            &[
                "........yaaay.", //
                ".......yaoooay", //
                ".......aoggoa.", //
                "......yaggggkg", //
                "......aoggggYY", //
                "..ff...aogggr.", //
                ".fggfffffgga..", //
                "fgLLggggggg...", //
                "..fggLLLLgg...", //
                "...ffggggff...", //
                "...f.f..f.f...", //
                "..ff.f.ff.f...", //
            ],
        ],
    },
    SpriteDef {
        name: "puffer",
        outline: true,
        frames: &[
            &[
                "............", //
                "............", //
                "....PPPP....", //
                "...PhhhhP...", //
                "..PhHhhhhP..", //
                "..PhhkhkhP..", //
                "..PhhhhhhP..", //
                "..PuhhhhuP..", //
                "...PuuuuP...", //
                "....PPPP....", //
                "....u..u....", //
                "............", //
            ],
            &[
                "............", //
                "....PPPP....", //
                "...PhhhhP...", //
                "..PhHHhhhP..", //
                ".PhhHhhhhhP.", //
                ".PhhhkhhkhP.", //
                ".PhhhhhhhhP.", //
                ".PuhhhhhhuP.", //
                "..PuhhhhuP..", //
                "...PuuuuP...", //
                "....u..u....", //
                "............", //
            ],
            &[
                ".....H......", //
                "..H.PPPP.H..", //
                "...PhhhhP...", //
                "..PhHHhhhP..", //
                "HPhhHhhhhhPH", //
                ".PhhhkhhkhP.", //
                ".PhhhhhhhhP.", //
                ".PuhhhhhhuP.", //
                "H.PuhhhhuP.H", //
                "...PuuuuP...", //
                "..H.u..u.H..", //
                "............", //
            ],
        ],
    },
    SpriteDef {
        name: "slime",
        outline: true,
        frames: &[
            &[
                "............", //
                "....uuuu....", //
                "..uuPPPPuu..", //
                ".uPPHPPPPPu.", //
                ".uPHPPPPPPu.", //
                "uPPPkPPkPPPu", //
                "uPPPkPPkPPPu", //
                "uPPPPPPPPPPu", //
                "uuPPPPPPPPuu", //
                ".uuuuuuuuuu.", //
            ],
            &[
                "............", //
                "............", //
                "...uuuuuu...", //
                ".uuPPHPPPuu.", //
                "uPPHPPPPPPPu", //
                "uPPPkPPkPPPu", //
                "uPPPkPPkPPPu", //
                "uPPPPPPPPPPu", //
                "uuPPPPPPPPuu", //
                "uuuuuuuuuuuu", //
            ],
        ],
    },
    SpriteDef {
        name: "slimelet",
        outline: true,
        frames: &[
            &[
                "..uuuu..", //
                ".uPHPPu.", //
                "uPkPPkPu", //
                "uPkPPkPu", //
                "uPPPPPPu", //
                ".uuuuuu.", //
            ],
            &[
                "........", //
                ".uuuuuu.", //
                "uPHPPPPu", //
                "uPkPPkPu", //
                "uPPPPPPu", //
                "uuuuuuuu", //
            ],
        ],
    },
    SpriteDef {
        name: "mole",
        outline: true,
        frames: &[
            &[
                ".............", //
                "....zzzzz....", //
                "..zzZZZZZzz..", //
                ".zZZZZZZZZkz.", //
                ".zZZxZZZZZZzh", //
                "zZZxxZZZZZZhh", //
                "zZZZZZZZZZZz.", //
                ".zzZZZZZZZzpp", //
                "..zpp.zzzppw.", //
                "..pw.....w...", //
            ],
            &[
                ".............", //
                "....zzzzz....", //
                "..zzZZZZZzz..", //
                ".zZZZZZZZZkz.", //
                ".zZZxZZZZZZzh", //
                "zZZxxZZZZZZhh", //
                "zZZZZZZZZZZz.", //
                ".zzZZZZZZZzpp", //
                "...zpp.zppw..", //
                "...pw...pw...", //
            ],
        ],
    },
    SpriteDef {
        name: "shellback",
        outline: true,
        frames: &[
            &[
                "....fffff......", //
                "..ffglllgff....", //
                ".fgllgggllgf...", //
                ".fggfgggfggfxx.", //
                "fgllgfgfgllgxkx", //
                "fggggfgfggggxxZ", //
                "fffffffffffffZ.", //
                ".ZxZ.....ZxZ...", //
                ".ZZ......ZZ....", //
            ],
            &[
                "....fffff......", //
                "..ffglllgff....", //
                ".fgllgggllgf...", //
                ".fggfgggfggfxx.", //
                "fgllgfgfgllgxkx", //
                "fggggfgfggggxxZ", //
                "fffffffffffffZ.", //
                "..ZxZ....ZxZ...", //
                "..ZZ......ZZ...", //
            ],
        ],
    },
    SpriteDef {
        name: "broodmother",
        outline: true,
        frames: &[
            &[
                "v..v......v..v", //
                ".v..v....v..v.", //
                "..v.vxYxxv.v..", //
                "...vxYxxYxv...", //
                "vv.uxxxYxxu.vv", //
                "..vuPPPPPPuv..", //
                ".v.uPPrrPPu.v.", //
                "v..uPPrrPPu..v", //
                "...uPPPPPPu...", //
                "..v.uPPPPu.v..", //
                ".v...hkkh...v.", //
                "v.....hh.....v", //
            ],
            &[
                ".v..v....v..v.", //
                "..v.v....v.v..", //
                "...vvxYxxvv...", //
                "v..vxYYxYxv..v", //
                ".v.uxxxYxxu.v.", //
                "..vuPPPPPPuv..", //
                "vv.uPPrrPPu.vv", //
                "...uPPrrPPu...", //
                "..vuPPPPPPuv..", //
                ".v..uPPPPu..v.", //
                "v....hkkh....v", //
                "......hh......", //
            ],
        ],
    },
    SpriteDef {
        name: "boomer",
        outline: true,
        frames: &[
            &[
                ".......y...", //
                "......ay...", //
                ".....z.....", //
                "...sszss...", //
                "..svvssss..", //
                ".svesssssd.", //
                ".svssosossd", //
                ".sssssssssd", //
                ".dssrrrrsdd", //
                "..dsssssdd.", //
                "...ddddd...", //
                "..n.n.n.n..", //
            ],
            &[
                "......Y.y..", //
                ".......a...", //
                ".....z.....", //
                "...sszss...", //
                "..svvssss..", //
                ".svesssssd.", //
                ".svssasassd", //
                ".sssssssssd", //
                ".dssoooosdd", //
                "..dsssssdd.", //
                "...ddddd...", //
                "...n.n.n.n.", //
            ],
        ],
    },
    SpriteDef {
        name: "mite",
        outline: true,
        frames: &[
            &[
                "..pp..", //
                ".crrc.", //
                "crorrk", //
                ".cmmc.", //
                ".n..n.", //
            ],
            &[
                ".p..p.", //
                ".crrc.", //
                "crorrk", //
                ".cmmc.", //
                "n....n", //
            ],
        ],
    },
    SpriteDef {
        name: "ptero",
        outline: true,
        frames: &[
            &[
                "v.............v", //
                "vv...........vv", //
                "vev...zzz...vev", //
                ".vev.zZxZz.vev.", //
                ".vvevZkZkZvevv.", //
                "..vvvZZZZZvvv..", //
                "....vZxYxZv....", //
                ".....zYYYz.....", //
                "......a.a......", //
            ],
            &[
                "...............", //
                "...............", //
                "......zzz......", //
                ".....zZxZz.....", //
                "vvvevZkZkZvevvv", //
                "evvvvZZZZZvvvve", //
                ".e..vZxYxZv..e.", //
                ".....zYYYz.....", //
                "......a.a......", //
            ],
            &[
                "...............", //
                "...............", //
                "......zzz......", //
                ".....zZxZz.....", //
                "....vZkZkZv....", //
                "..vvvZZZZZvvv..", //
                ".vevvZxYxZvvev.", //
                "vev..zYYYz..vev", //
                "v.....a.a.....v", //
            ],
            &[
                "...............", //
                "...............", //
                "......zzz......", //
                ".....zZxZz.....", //
                "vvvevZkZkZvevvv", //
                "evvvvZZZZZvvvve", //
                ".e..vZxYxZv..e.", //
                ".....zYYYz.....", //
                "......a.a......", //
            ],
        ],
    },
    SpriteDef {
        name: "ghost",
        outline: true,
        frames: &[
            &[
                "...ppppp...", //
                "..pwwwwwp..", //
                ".pwwwwwwwp.", //
                ".pwkkwkkwp.", //
                ".pwkiwkiwp.", //
                ".pwwwkwwwp.", //
                "..pwwwwwp..", //
                "..pwkwkwp..", //
                "..pIpIpIp..", //
                "...IpIpI...", //
                "....I.I....", //
                "...I...I...", //
            ],
            &[
                "...ppppp...", //
                "..pwwwwwp..", //
                ".pwwwwwwwp.", //
                ".pwkkwkkwp.", //
                ".pwkiwkiwp.", //
                ".pwwwkwwwp.", //
                "..pwwwwwp..", //
                "..pwkwkwp..", //
                "..IpIpIpI..", //
                "...pIpIp...", //
                "...I.I.....", //
                "....I...I..", //
            ],
        ],
    },
    SpriteDef {
        name: "eyestalk",
        outline: true,
        frames: &[
            &[
                "...ppppp...", //
                "..pwwwwwp..", //
                ".pwwwwrrwp.", //
                ".pwwwrkkrp.", //
                ".pwwwrkkrp.", //
                ".pwwwwrrwp.", //
                "..cpwwwpc..", //
                "...ppppp...", //
                ".....f.....", //
                "....ff.....", //
                "....f......", //
                "....ff.....", //
                "..gfffgg...", //
                ".ggffggfg..", //
            ],
            &[
                "...ppppp...", //
                "..pwwwwwp..", //
                ".pwwwwrrwp.", //
                ".pwwwrkkrp.", //
                ".pwwwrkkrp.", //
                ".pwwwwrrwp.", //
                "..cpwwwpc..", //
                "...ppppp...", //
                ".....f.....", //
                ".....ff....", //
                "......f....", //
                ".....ff....", //
                "..gfffgg...", //
                ".ggffggfg..", //
            ],
        ],
    },
    SpriteDef {
        name: "golem",
        outline: true,
        frames: &[
            &[
                "....ZZZZZZ....", //
                "...ZxxxxxxZ...", //
                "..ZxxZZZZxxZ..", //
                "..ZxiiZZiixZ..", //
                "..ZxxxZZxxxZ..", //
                ".ZZZxxzzxxZZZ.", //
                "ZfxZZZZZZZZxfZ", //
                "ZxxZxxiixxZxxZ", //
                "ZxZZxxxxxxZZxZ", //
                "ZxZ.ZxxxxZ.ZxZ", //
                "ZZZ.ZZZZZZ.ZZZ", //
                "....ZxZZxZ....", //
                "...ZxxZ.ZxxZ..", //
                "...ZZZ...ZZZ..", //
            ],
            &[
                "....ZZZZZZ....", //
                "...ZxxxxxxZ...", //
                "..ZxxZZZZxxZ..", //
                "..ZxIIZZIIxZ..", //
                "..ZxxxZZxxxZ..", //
                ".ZZZxxzzxxZZZ.", //
                "ZfxZZZZZZZZxfZ", //
                "ZxxZxxIIxxZxxZ", //
                "ZxZZxxxxxxZZxZ", //
                "ZxZ.ZxxxxZ.ZxZ", //
                "ZZZ.ZZZZZZ.ZZZ", //
                "....ZxZZxZ....", //
                "..ZxxZ...ZxxZ.", //
                "..ZZZ.....ZZZ.", //
            ],
        ],
    },
    // Bosses.
    SpriteDef {
        name: "mire_queen",
        outline: true,
        frames: &[
            &[
                "........y..y..y.........", //
                "........yayyayya........", //
                "........yyyyyyyy........", //
                "......uuPPPPPPPPuu......", //
                "....uuPPPPPPHHPPPPuu....", //
                "...uPPPPPPPPPHHPPPPPu...", //
                "..uPPPPPPPPPPPPHPPPPPu..", //
                "..uPPwwwPPPPPPwwwPPPPu..", //
                ".uPPwHkwPPPPPPwHkwPPPPu.", //
                ".uPPwkkwPPPPPPwkkwPPPPu.", //
                ".uPPPwwPPPPPPPPwwPPPPPu.", //
                "uPPPPPPPPPPPPPPPPPPPPPPu", //
                "uPPPPPPmmmmmmmmmPPPPPPPu", //
                "uPPPPPPmwmmwmmwmPPPPPPPu", //
                "uPPPPPPPmmmmmmmPPPPPPPPu", //
                "uuPPPPPPPPPPPPPPPPPPPPuu", //
                ".uuPPPPPPPPPPPPPPPPPPuu.", //
                "u.uuuPPPPPPPPPPPPPPuuu.u", //
                "...u.uuuuuuuuuuuuuu.u...", //
                "........u.....u.........", //
            ],
            &[
                "........................", //
                "........y..y..y.........", //
                "........yayyayya........", //
                "........yyyyyyyy........", //
                "....uuuuPPPPPPPPuuuu....", //
                "..uuPPPPPPPPPHHPPPPPuu..", //
                ".uPPPPPPPPPPPPPHPPPPPPu.", //
                ".uPPPwwwPPPPPPwwwPPPPPu.", //
                "uPPPwHkwPPPPPPwHkwPPPPPu", //
                "uPPPwkkwPPPPPPwkkwPPPPPu", //
                "uPPPPwwPPPPPPPPwwPPPPPPu", //
                "uPPPPPPPmmmmmmmmmPPPPPPu", //
                "uPPPPPPPmwmmwmmwmPPPPPPu", //
                "uPPPPPPPPmmmmmmmPPPPPPPu", //
                "uuPPPPPPPPPPPPPPPPPPPPuu", //
                ".uuPPPPPPPPPPPPPPPPPPuu.", //
                "..uuuPPPPPPPPPPPPPPuuu..", //
                ".u..uuuuuuuuuuuuuuu..u..", //
                "......u.......u.....u...", //
                "........................", //
            ],
        ],
    },
    SpriteDef {
        name: "colossus",
        outline: true,
        frames: &[
            &[
                "............eeee........", //
                "..........eeppppe.......", //
                ".........eppkppkpe...ww.", //
                "........eppppppppe..wwe.", //
                "........epkppkpppwwwwe..", //
                "..pppp..eppppppwwwwwwe..", //
                ".pwwwwpp.eeppwwwkowwwwe.", //
                "pwnwnwnwp..epwwwkkwwwwwp", //
                "pwnwnwnwnwp.epwwwwwwwpe.", //
                "ewnwnwnwnwwpppwwwwpppe..", //
                ".ewnwnwnwnwweeeepppe....", //
                "..eewwwwwwweee..........", //
                "...pw.pw...pw.pw........", //
                "...ew.ew...ew.ew........", //
                "..eww.eww.eww.eww.......", //
            ],
            &[
                "............eeee........", //
                "..........eeppppe.......", //
                ".........eppkppkpe...ww.", //
                "........eppppppppe..wwe.", //
                "........epkppkpppwwwwe..", //
                "..pppp..eppppppwwwwwwe..", //
                ".pwwwwpp.eeppwwwkawwwwe.", //
                "pwnwnwnwp..epwwwkkwwwwwp", //
                "pwnwnwnwnwp.epwwwwwwwpe.", //
                "ewnwnwnwnwwpppwwwwpppe..", //
                ".ewnwnwnwnwweeeepppe....", //
                "..eewwwwwwweee..........", //
                "..pw..pw...pw..pw.......", //
                "..ew...ew..ew...ew......", //
                ".eww..eww.eww..eww......", //
            ],
        ],
    },
    SpriteDef {
        name: "wraith",
        outline: true,
        frames: &[
            &[
                ".........NNNN.........", //
                ".......NNbbbbNN.......", //
                "......NbbBBBBbbN......", //
                ".....NbBBNNNNBBbN.....", //
                ".....NbBNIIIINBbN.....", //
                "....NbBNIiIIiINBbN....", //
                "....NbBNIkIIkINBbN....", //
                "....NbBNNIIIINNBbN....", //
                "...NbbBBNNNNNNBBbbN...", //
                "..NbbBBBBBBBBBBBBbbN..", //
                ".iNbBBBBbBBBBbBBBBbNi.", //
                "iINbBBBbBBBBBBbBBBbNIi", //
                ".iNbbBBbBBBBBBbBBbbNi.", //
                "..NbbBbBBBBBBBBbBbbN..", //
                "..NbbbbBBBBBBBBbbbbN..", //
                "..NNbbbbBBBBBBbbbbNN..", //
                "...NNbbbbbbbbbbbbNN...", //
                "...N.NbbNbbbbNbbN.N...", //
                "......NN.NbbN.NN......", //
                "..........NN..........", //
            ],
            &[
                ".........NNNN.........", //
                ".......NNbbbbNN.......", //
                "......NbbBBBBbbN......", //
                ".....NbBBNNNNBBbN.....", //
                ".....NbBNIIIINBbN.....", //
                "....NbBNIiIIiINBbN....", //
                "....NbBNIkIIkINBbN....", //
                "....NbBNNIIIINNBbN....", //
                "...NbbBBNNNNNNBBbbN...", //
                "i.NbbBBBBBBBBBBBBbbN.i", //
                "IiNbBBBBbBBBBbBBBBbNiI", //
                ".iNbBBBbBBBBBBbBBBbNi.", //
                "..NbbBBbBBBBBBbBBbbN..", //
                "..NbbBbBBBBBBBBbBbbN..", //
                "..NbbbbBBBBBBBBbbbbN..", //
                "..NNbbbbBBBBBBbbbbNN..", //
                "...NNbbbbbbbbbbbbNN...", //
                "....NbbN.NbbN.NbbN....", //
                "....NN...NbbN..NN.....", //
                ".........N..N.........", //
            ],
            &[
                ".........NNNN.........", //
                ".......NNbbbbNN.......", //
                "......NbbBBBBbbN......", //
                ".....NbBBNNNNBBbN.....", //
                ".....NbBNIIIINBbN.....", //
                "....NbBNIiIIiINBbN....", //
                "....NbBNIkIIkINBbN....", //
                "....NbBNNIIIINNBbN....", //
                "...NbbBBNNNNNNBBbbN...", //
                "..NbbBBBBBBBBBBBBbbN..", //
                "..NbBBBBbBBBBbBBBBbN..", //
                ".iNbBBBbBBBBBBbBBBbNi.", //
                "iINbbBBbBBBBBBbBBbbNIi", //
                ".iNbbBbBBBBBBBBbBbbNi.", //
                "..NbbbbBBBBBBBBbbbbN..", //
                "..NNbbbbBBBBBBbbbbNN..", //
                "...NNbbbbbbbbbbbbNN...", //
                "...NbbN.NbbbbN.NbbN...", //
                "....NN...NbbN...NN....", //
                "..........NN..........", //
            ],
        ],
    },
    SpriteDef {
        name: "sandmaw",
        outline: true,
        frames: &[
            &[
                "......ZZZZZZZZ......", //
                "....ZZxxxxxxxxZZ....", //
                "...ZxxxxxxxxxxxxZ...", //
                "..ZxxwZZZZZZZZwxxZ..", //
                "..ZxwZmmmmmmmmZwxZ..", //
                ".ZxxZmwcccccwmmZxxZ.", //
                ".ZxZmmccrrrcccmmZxZ.", //
                ".ZxZmwcrrmrrcwmmZxZ.", //
                ".ZxZmmccrrrcccmmZxZ.", //
                ".ZxxZmwcccccwmmZxxZ.", //
                "..ZxwZmmmmmmmmZwxZ..", //
                "..ZxxwZZZZZZZZwxxZ..", //
                "...ZxxxxxxxxxxxxZ...", //
                "....ZZZxxxxxxZZZ....", //
                "....zZZZZZZZZZZz....", //
                "....zxxxxxxxxxxz....", //
                "....zZZZZZZZZZZz....", //
                "...zzxxxxxxxxxxzz...", //
                "..zzzZZZZZZZZZZzzz..", //
            ],
            &[
                "....................", //
                "......ZZZZZZZZ......", //
                "....ZZxxxxxxxxZZ....", //
                "...ZxxxwZZZZwxxxZ...", //
                "..ZxxwZmmmmmmZwxxZ..", //
                "..ZxwZmwccccwmZwxZ..", //
                ".ZxxZmmcrrrrcmmZxxZ.", //
                ".ZxZmwcrrmmrrcwmZxZ.", //
                ".ZxxZmmcrrrrcmmZxxZ.", //
                "..ZxwZmwccccwmZwxZ..", //
                "..ZxxwZmmmmmmZwxxZ..", //
                "...ZxxxwZZZZwxxxZ...", //
                "....ZxxxxxxxxxxZ....", //
                "....ZZZxxxxxxZZZ....", //
                "....zZZZZZZZZZZz....", //
                "....zxxxxxxxxxxz....", //
                "....zZZZZZZZZZZz....", //
                "...zzxxxxxxxxxxzz...", //
                "..zzzZZZZZZZZZZzzz..", //
            ],
        ],
    },
    SpriteDef {
        name: "hive_eye",
        outline: true,
        frames: &[
            &[
                "......uuuuuuuu......", //
                "....uuPPPPPPPPuu....", //
                "...uPPppppppppPPu...", //
                "..uPppwwwwwwwwppPu..", //
                ".uPpwwwwwwwwwwwwpPu.", //
                ".uPwwwwwrrrrwwwwwPu.", //
                "uPpwwwwrrccrrwwwwpPu", //
                "uPwwwwrrckkcrrwwwwPu", //
                "uPwwwwrcckkkcrwwwwPu", //
                "uPwwwwrrckkcrrwwwwPu", //
                "uPpwwwwrrccrrwwwwpPu", //
                ".uPwwwwwrrrrwwwwwPu.", //
                ".uPpwwwwwwwwwwwwpPu.", //
                "..uPppwwwwwwwwppPu..", //
                "...uPPppppppppPPu...", //
                "..h.uuPPPPPPPPuu.h..", //
                ".h..h.uuuuuuuu.h..h.", //
                ".h.h..h..h..h..h.h..", //
                "h..h.h..h....h..h..h", //
            ],
            &[
                "......uuuuuuuu......", //
                "....uuPPPPPPPPuu....", //
                "...uPPppppppppPPu...", //
                "..uPppwwwwwwwwppPu..", //
                ".uPpwwwwwwwwwwwwpPu.", //
                ".uPwwwwwwwrrrrwwwPu.", //
                "uPpwwwwwwrrccrrwwpPu", //
                "uPwwwwwwrrckkcrrwwPu", //
                "uPwwwwwwrcckkkcrwwPu", //
                "uPwwwwwwrrckkcrrwwPu", //
                "uPpwwwwwwrrccrrwwpPu", //
                ".uPwwwwwwwrrrrwwwPu.", //
                ".uPpwwwwwwwwwwwwpPu.", //
                "..uPppwwwwwwwwppPu..", //
                "...uPPppppppppPPu...", //
                "...huuPPPPPPPPuuh...", //
                "..h.h.uuuuuuuu.h.h..", //
                "..h..h.h..h.h..h..h.", //
                ".h..h..h..h..h..h..h", //
            ],
            &[
                "......uuuuuuuu......", //
                "....uuPPPPPPPPuu....", //
                "...uPPppppppppPPu...", //
                "..uPppwwwwwwwwppPu..", //
                ".uPpwwwwwwwwwwwwpPu.", //
                ".uPwwwwwrrrrwwwwwPu.", //
                "uPpwwwwrrccrrwwwwpPu", //
                "uPwwwwrrckkcrrwwwwPu", //
                "uPwwwwrcckkkcrwwwwPu", //
                "uPwwwwrrckkcrrwwwwPu", //
                "uPpwwwwrrccrrwwwwpPu", //
                ".uPwwwwwrrrrwwwwwPu.", //
                ".uPpwwwwwwwwwwwwpPu.", //
                "..uPppwwwwwwwwppPu..", //
                "...uPPppppppppPPu...", //
                "..h.uuPPPPPPPPuu.h..", //
                ".h..h.uuuuuuuu.h..h.", //
                ".h.h..h..h..h..h.h..", //
                "h..h.h..h....h..h..h", //
            ],
            &[
                "......uuuuuuuu......", //
                "....uuPPPPPPPPuu....", //
                "...uPPppppppppPPu...", //
                "..uPppwwwwwwwwppPu..", //
                ".uPpwwwwwwwwwwwwpPu.", //
                ".uPwwwwwwwrrrrwwwPu.", //
                "uPpwwwwwwrrccrrwwpPu", //
                "uPwwwwwwrrckkcrrwwPu", //
                "uPwwwwwwrcckkkcrwwPu", //
                "uPwwwwwwrrckkcrrwwPu", //
                "uPpwwwwwwrrccrrwwpPu", //
                ".uPwwwwwwwrrrrwwwPu.", //
                ".uPpwwwwwwwwwwwwpPu.", //
                "..uPppwwwwwwwwppPu..", //
                "...uPPppppppppPPu...", //
                "...huuPPPPPPPPuuh...", //
                "..h.h.uuuuuuuu.h.h..", //
                "..h..h.h..h.h..h..h.", //
                ".h..h..h..h..h..h..h", //
            ],
            &[
                "......uuuuuuuu......", //
                "....uuPPPPPPPPuu....", //
                "...uPPppppppppPPu...", //
                "..uPppwwwwwwwwppPu..", //
                ".uPpwwwwwwwwwwwwpPu.", //
                ".uPwwwwwrrrrwwwwwPu.", //
                "uPpwwwwrrccrrwwwwpPu", //
                "uPwwwwrrckkcrrwwwwPu", //
                "uPwwwwrcckkkcrwwwwPu", //
                "uPwwwwrrckkcrrwwwwPu", //
                "uPpwwwwrrccrrwwwwpPu", //
                ".uPwwwwwrrrrwwwwwPu.", //
                ".uPpwwwwwwwwwwwwpPu.", //
                "..uPppwwwwwwwwppPu..", //
                "...uPPppppppppPPu...", //
                "..h.uuPPPPPPPPuu.h..", //
                ".h..h.uuuuuuuu.h..h.", //
                ".h.h..h..h..h..h.h..", //
                "h..h.h..h....h..h..h", //
            ],
            &[
                "......uuuuuuuu......", //
                "....uuPPPPPPPPuu....", //
                "...uPPPPPPPPPPPPu...", //
                "..uPPPPPPPPPPPPPPu..", //
                ".uPPPPPPPPPPPPPPPPu.", //
                ".uPPPPPPPPPPPPPPPPu.", //
                "uPPPPPPPPPPPPPPPPPPu", //
                "uPpppppppppppppppPPu", //
                "uPPwwwwwwwwwwwwwwPPu", //
                "uPppppppppppppppppPu", //
                "uPPPPPPPPPPPPPPPPPPu", //
                ".uPPPPPPPPPPPPPPPPu.", //
                ".uPPPPPPPPPPPPPPPPu.", //
                "..uPPPPPPPPPPPPPPu..", //
                "...uPPPPPPPPPPPPu...", //
                "..h.uuPPPPPPPPuu.h..", //
                ".h..h.uuuuuuuu.h..h.", //
                ".h.h..h..h..h..h.h..", //
                "h..h.h..h....h..h..h", //
            ],
        ],
    },
    // Enemy projectiles.
    SpriteDef {
        name: "spit",
        outline: true,
        frames: &[&[
            ".l.", //
            "lLl", //
            ".g.", //
        ]],
    },
    SpriteDef {
        name: "glob",
        outline: true,
        frames: &[
            &[
                ".PP.", //
                "PHhP", //
                "PhhP", //
                ".PP.", //
            ],
            &[
                ".hh.", //
                "hHHh", //
                "hHhh", //
                ".hh.", //
            ],
        ],
    },
    SpriteDef {
        name: "orb",
        outline: true,
        frames: &[
            &[
                ".ll.", //
                "lLwl", //
                "lLLl", //
                ".ll.", //
            ],
            &[
                ".gg.", //
                "glLg", //
                "gLlg", //
                ".gg.", //
            ],
        ],
    },
    SpriteDef {
        name: "shard",
        outline: true,
        frames: &[
            &[
                ".w.", //
                "wpw", //
                ".w.", //
            ],
            &[
                ".p.", //
                "pwp", //
                ".p.", //
            ],
        ],
    },
    SpriteDef {
        name: "needle",
        outline: true,
        frames: &[
            &[
                ".o.", //
                "oYo", //
                ".o.", //
            ],
            &[
                ".r.", //
                "rar", //
                ".r.", //
            ],
        ],
    },
    SpriteDef {
        name: "grit",
        outline: true,
        frames: &[
            &[
                ".w.", //
                "wrw", //
                ".w.", //
            ],
            &[
                ".p.", //
                "pcp", //
                ".p.", //
            ],
        ],
    },
    SpriteDef {
        name: "spore",
        outline: true,
        frames: &[
            &[
                ".h.", //
                "hHh", //
                ".h.", //
            ],
            &[
                ".H.", //
                "HwH", //
                ".H.", //
            ],
        ],
    },
    // Burrowed enemies.
    SpriteDef {
        name: "mound",
        outline: true,
        frames: &[
            &[
                "...zZZz...", //
                "..zZxZZz..", //
                ".zZZZZzZz.", //
                "zZzZZZZZzz", //
                ".zzzzzzzz.", //
            ],
            &[
                "..x.zZz...", //
                "..zZZxZz..", //
                ".zZxZZZZz.", //
                "zZZZzZZZZz", //
                ".zzzzzzzz.", //
            ],
        ],
    },
    SpriteDef {
        name: "mound_big",
        outline: true,
        frames: &[
            &[
                "......zZZZz.......", //
                "....zZZxZZZzz.....", //
                "..zzZZZZZxZZZZz...", //
                ".zZZxZZZZZZZzZZzz.", //
                "zZZZZZZzZZZZZZZZZz", //
                "zZzZZZZZZZZZzZZZZz", //
                ".zzzzzzzzzzzzzzzz.", //
            ],
            &[
                "....x..zZZz....x..", //
                "....zZZZZxZzz.....", //
                "..zzZxZZZZZZZZz...", //
                ".zZZZZZZxZZZzZZzz.", //
                "zZZzZZZZZZZZZZxZZz", //
                "zZZZZZzZZZZZZZZZZz", //
                ".zzzzzzzzzzzzzzzz.", //
            ],
        ],
    },
];
