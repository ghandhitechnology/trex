//! Character and weapon art. Characters face right; the renderer flips them.

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
];
