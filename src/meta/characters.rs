use std::collections::BTreeMap;

use serde::Deserialize;

use crate::items::{Stat, StatMod, Trigger};
use crate::render::sprite::SpriteId;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CharacterDef {
    pub id: String,
    pub name: String,
    pub desc: String,
    /// Walk animation.
    pub sprite: String,
    /// Idle animation. Optional; without it the hero stands on the first walk frame.
    #[serde(default)]
    pub idle: String,
    /// Pose played once per shot of the hero's weapon. Optional.
    #[serde(default)]
    pub attack: String,
    /// Pose held through a dash. Optional.
    #[serde(default)]
    pub dash: String,
    /// Art row where the legs start. While running, the attack pose keeps the
    /// stride's legs from this row down. 0 plays the whole pose.
    #[serde(default)]
    pub legs: i32,
    /// Where the hero's shots leave the sprite (mouth, horn, claw), in pixels
    /// from the sprite center with the hero facing right.
    #[serde(default)]
    pub muzzle: (f32, f32),
    /// Meta currency cost to unlock. 0 means available from the start.
    #[serde(default)]
    pub unlock: u32,
    pub weapon: WeaponDef,
    /// Overrides for engine default stats.
    #[serde(default)]
    pub base: BTreeMap<Stat, f32>,
    /// Passive stat modifiers (same rules as items).
    #[serde(default)]
    pub stats: Vec<StatMod>,
    /// Passive triggers (same rules as items).
    #[serde(default)]
    pub triggers: Vec<Trigger>,
    #[serde(skip)]
    pub sprite_id: SpriteId,
    #[serde(skip)]
    pub idle_id: Option<SpriteId>,
    #[serde(skip)]
    pub attack_id: Option<SpriteId>,
    #[serde(skip)]
    pub dash_id: Option<SpriteId>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct WeaponDef {
    /// Projectile sprite name.
    pub shot: String,
    #[serde(default)]
    pub pattern: Pattern,
    #[serde(skip)]
    pub shot_id: SpriteId,
}

#[derive(Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    /// Fan of `Shots` aimed at the nearest enemy.
    #[default]
    Aimed,
    /// `Shots` spread evenly in a full circle, no target needed.
    Radial,
}
