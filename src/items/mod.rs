//! Items and the composable effect model: stat modifiers plus triggered actions.
//! Characters use the same `StatMod` and `Trigger` types for their passives.

pub mod effects;
pub mod sprites;

use serde::Deserialize;

use crate::content::Content;
use crate::engine::Rng;
use crate::meta::characters::CharacterDef;
use crate::render::sprite::SpriteId;

/// Every tunable number on the player. Items and characters modify these.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stat {
    /// Hit points (whole hearts are 2).
    MaxHp,
    /// Move speed, px/s.
    Speed,
    /// Base damage per shot. Action damage is a ratio of this.
    Damage,
    /// Volleys per second.
    FireRate,
    /// Projectiles per volley.
    Shots,
    /// Degrees between projectiles in a volley.
    Spread,
    /// Projectile speed, px/s.
    ShotSpeed,
    /// Projectile size multiplier.
    ShotSize,
    /// Projectile travel distance, px.
    Range,
    /// Enemies a projectile passes through.
    Pierce,
    /// Times a projectile redirects to a new enemy after a hit.
    Bounce,
    /// Projectile turn rate toward enemies, rad/s.
    Homing,
    /// Push on hit, px/s.
    Knockback,
    /// Crit chance, 0..1.
    Crit,
    /// Crit damage multiplier.
    CritDamage,
    /// Gem magnet radius, px.
    Pickup,
    /// XP multiplier.
    XpGain,
    /// HP restored per minute.
    Regen,
    /// Chance to ignore a hit, 0..0.6.
    Dodge,
    /// Seconds between dashes.
    DashCooldown,
    /// Radius multiplier for explosions, novas, and shockwaves.
    Area,
    /// Duration multiplier for burn and slow.
    Duration,
}

pub const STAT_COUNT: usize = 22;

impl Stat {
    pub const ALL: [Stat; STAT_COUNT] = [
        Stat::MaxHp,
        Stat::Speed,
        Stat::Damage,
        Stat::FireRate,
        Stat::Shots,
        Stat::Spread,
        Stat::ShotSpeed,
        Stat::ShotSize,
        Stat::Range,
        Stat::Pierce,
        Stat::Bounce,
        Stat::Homing,
        Stat::Knockback,
        Stat::Crit,
        Stat::CritDamage,
        Stat::Pickup,
        Stat::XpGain,
        Stat::Regen,
        Stat::Dodge,
        Stat::DashCooldown,
        Stat::Area,
        Stat::Duration,
    ];

    /// Engine default before character overrides.
    pub fn default_value(self) -> f32 {
        match self {
            Stat::MaxHp => 6.0,
            Stat::Speed => 68.0,
            Stat::Damage => 6.0,
            Stat::FireRate => 2.0,
            Stat::Shots => 1.0,
            Stat::Spread => 12.0,
            Stat::ShotSpeed => 170.0,
            Stat::ShotSize => 1.0,
            Stat::Range => 140.0,
            Stat::Pierce | Stat::Bounce | Stat::Homing => 0.0,
            Stat::Knockback => 60.0,
            Stat::Crit => 0.05,
            Stat::CritDamage => 2.0,
            Stat::Pickup => 28.0,
            Stat::XpGain => 1.0,
            Stat::Regen | Stat::Dodge => 0.0,
            Stat::DashCooldown => 1.4,
            Stat::Area | Stat::Duration => 1.0,
        }
    }

    /// Valid range after all modifiers.
    pub fn limits(self) -> (f32, f32) {
        match self {
            Stat::MaxHp => (1.0, 40.0),
            Stat::Speed => (20.0, 200.0),
            Stat::FireRate => (0.2, 20.0),
            Stat::Shots => (1.0, 16.0),
            Stat::ShotSize => (0.4, 4.0),
            Stat::Crit => (0.0, 1.0),
            Stat::CritDamage => (1.0, 10.0),
            Stat::Dodge => (0.0, 0.6),
            Stat::DashCooldown => (0.25, 10.0),
            Stat::Area | Stat::Duration => (0.25, 5.0),
            _ => (0.0, f32::MAX),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats(pub [f32; STAT_COUNT]);

impl Stats {
    pub fn defaults() -> Self {
        Stats(Stat::ALL.map(Stat::default_value))
    }

    pub fn get(&self, s: Stat) -> f32 {
        self.0[s as usize]
    }

    /// Whole-number view for count stats (Shots, Pierce, Bounce, MaxHp).
    pub fn count(&self, s: Stat) -> u32 {
        self.get(s).max(0.0).floor() as u32
    }

    pub fn set(&mut self, s: Stat, v: f32) {
        self.0[s as usize] = v;
    }
}

/// `add` is summed onto the base, `mul` is summed into one multiplier:
/// final = (base + Σadd) × (1 + Σmul), then clamped to the stat's limits.
#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StatMod {
    pub stat: Stat,
    #[serde(default)]
    pub add: f32,
    #[serde(default)]
    pub mul: f32,
}

pub fn apply_mods<'a>(base: &Stats, mods: impl IntoIterator<Item = &'a StatMod>) -> Stats {
    let mut add = [0.0f32; STAT_COUNT];
    let mut mul = [0.0f32; STAT_COUNT];
    for m in mods {
        add[m.stat as usize] += m.add;
        mul[m.stat as usize] += m.mul;
    }
    let mut out = *base;
    for s in Stat::ALL {
        let i = s as usize;
        let (lo, hi) = s.limits();
        out.0[i] = ((base.0[i] + add[i]) * (1.0 + mul[i]).max(0.0)).clamp(lo, hi);
    }
    out
}

/// Game events a trigger can listen to.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum On {
    /// A player projectile or effect damaged an enemy.
    Hit,
    /// A hit that crit.
    Crit,
    /// An enemy died.
    Kill,
    /// The player dashed.
    Dash,
    /// The player lost HP.
    Hurt,
    /// The player picked a level-up reward.
    LevelUp,
    /// Fires every `cooldown` seconds.
    Timer,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Trigger {
    pub on: On,
    #[serde(default = "one")]
    pub chance: f32,
    /// Minimum seconds between procs (the period for `Timer`).
    #[serde(default)]
    pub cooldown: f32,
    pub action: Action,
}

fn one() -> f32 {
    1.0
}

/// What a trigger does. `damage` values are ratios of the Damage stat; radii
/// scale with Area and durations with Duration.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum Action {
    /// Set the hit enemy on fire.
    Burn { dps: f32, secs: f32 },
    /// Slow the hit enemy by `amount` (0..1).
    Slow { amount: f32, secs: f32 },
    /// Damage every enemy in a radius around the event.
    Explode { radius: f32, damage: f32 },
    /// Fire `count` projectiles in a ring from the event position.
    Nova { count: u32, damage: f32 },
    /// Lightning that jumps between nearby enemies.
    Chain { jumps: u32, range: f32, damage: f32 },
    /// Fire `count` extra projectiles at the nearest enemies.
    Volley { count: u32, damage: f32 },
    /// Push enemies away from the event position.
    Shockwave { radius: f32, force: f32 },
    /// Restore HP.
    Heal { amount: i32 },
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Rarity {
    #[default]
    Common,
    Rare,
    Epic,
}

impl Rarity {
    pub fn weight(self) -> f32 {
        match self {
            Rarity::Common => 10.0,
            Rarity::Rare => 5.0,
            Rarity::Epic => 2.0,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub sprite: String,
    #[serde(default)]
    pub rarity: Rarity,
    #[serde(default = "default_stacks")]
    pub max_stacks: u32,
    /// Meta currency cost to unlock. 0 means available from the start.
    #[serde(default)]
    pub unlock: u32,
    #[serde(default)]
    pub stats: Vec<StatMod>,
    #[serde(default)]
    pub triggers: Vec<Trigger>,
    #[serde(skip)]
    pub sprite_id: SpriteId,
}

fn default_stacks() -> u32 {
    5
}

/// Items picked this run, in pick order.
#[derive(Clone, Debug, Default)]
pub struct Build {
    pub items: Vec<(usize, u32)>,
}

impl Build {
    pub fn stacks(&self, item: usize) -> u32 {
        self.items.iter().find(|(i, _)| *i == item).map_or(0, |(_, n)| *n)
    }

    pub fn add(&mut self, item: usize) {
        match self.items.iter_mut().find(|(i, _)| *i == item) {
            Some((_, n)) => *n += 1,
            None => self.items.push((item, 1)),
        }
    }

    /// Final stats: character base, then every modifier from the passive and item stacks.
    pub fn stats(&self, ch: &CharacterDef, content: &Content) -> Stats {
        let mut base = Stats::defaults();
        for (s, v) in &ch.base {
            base.set(*s, *v);
        }
        let item_mods =
            self.items.iter().flat_map(|&(i, n)| (0..n).flat_map(move |_| content.items[i].stats.iter()));
        apply_mods(&base, ch.stats.iter().chain(item_mods))
    }

    /// Every trigger instance. Each stack adds its triggers again.
    pub fn triggers(&self, ch: &CharacterDef, content: &Content) -> Vec<Trigger> {
        let mut out = ch.triggers.clone();
        for &(i, n) in &self.items {
            for _ in 0..n {
                out.extend(content.items[i].triggers.iter().cloned());
            }
        }
        out
    }
}

/// Up to `n` distinct item choices, weighted by rarity, skipping maxed and locked items.
pub fn roll_offer(
    content: &Content,
    build: &Build,
    unlocked: impl Fn(&ItemDef) -> bool,
    rng: &mut Rng,
    n: usize,
) -> Vec<usize> {
    let mut weights: Vec<f32> = content
        .items
        .iter()
        .enumerate()
        .map(|(i, it)| if unlocked(it) && build.stacks(i) < it.max_stacks { it.rarity.weight() } else { 0.0 })
        .collect();
    let mut picks = Vec::with_capacity(n);
    while picks.len() < n {
        let Some(i) = rng.weighted(&weights) else { break };
        weights[i] = 0.0;
        picks.push(i);
    }
    picks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_math_adds_then_multiplies_and_clamps() {
        let base = Stats::defaults();
        let mods = [
            StatMod { stat: Stat::Damage, add: 2.0, mul: 0.0 },
            StatMod { stat: Stat::Damage, add: 0.0, mul: 0.5 },
            StatMod { stat: Stat::Damage, add: 0.0, mul: -0.25 },
            StatMod { stat: Stat::Dodge, add: 5.0, mul: 0.0 },
            StatMod { stat: Stat::FireRate, add: 0.0, mul: -3.0 },
        ];
        let s = apply_mods(&base, &mods);
        let dmg = (Stat::Damage.default_value() + 2.0) * 1.25;
        assert!((s.get(Stat::Damage) - dmg).abs() < 1e-5);
        assert_eq!(s.get(Stat::Dodge), 0.6);
        assert_eq!(s.get(Stat::FireRate), Stat::FireRate.limits().0);
        assert_eq!(s.get(Stat::Speed), Stat::Speed.default_value());
    }

    #[test]
    fn stat_order_matches_all() {
        for (i, s) in Stat::ALL.iter().enumerate() {
            assert_eq!(*s as usize, i);
        }
    }
}
