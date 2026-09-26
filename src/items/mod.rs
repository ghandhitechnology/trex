//! Items and the composable effect model: stat modifiers, triggered actions,
//! item weapons, and named synergies. Characters use the same `StatMod` and
//! `Trigger` types for their passives.

pub mod draw;
pub mod effects;
pub mod sprites;
pub mod synergy;
pub mod weapons;

use serde::Deserialize;

use crate::content::Content;
use crate::engine::Rng;
use crate::meta::characters::CharacterDef;
use crate::render::sprite::{SpriteId, bank};
use synergy::SynergyDef;
use weapons::WeaponDef;

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

/// Whose stats a shot or effect uses: the hero's own (their weapon and
/// passive), or the hero-neutral item stats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Hero,
    Item,
}

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
    /// The player pressed Space (the dash key). Active items listen to this.
    Active,
}

/// What dealt a hit. Triggers can filter Hit, Crit and Kill events with `from`.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Source {
    /// The character's weapon, and anything untagged such as burn ticks.
    #[default]
    Main,
    Gun,
    Orbit,
    Zap,
    Beam,
    Mine,
    Aura,
    Meteor,
    Explode,
    Nova,
    Chain,
    Volley,
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
    /// Only fire for hits from this source.
    #[serde(default)]
    pub from: Option<Source>,
    pub action: Action,
    /// Item that owns this trigger, set at load. The HUD uses it for actives.
    #[serde(skip)]
    pub owner: Option<usize>,
}

fn one() -> f32 {
    1.0
}

/// What a trigger does. `damage` values are ratios of the Damage stat; radii
/// scale with Area and durations with Duration.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
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
    /// Push enemies away from the event position (negative force pulls).
    Shockwave { radius: f32, force: f32 },
    /// Restore HP.
    Heal { amount: i32 },
    /// Call `count` meteors down on random nearby enemies.
    Strike { count: u32, radius: f32, damage: f32 },
    /// Slow every enemy in a radius.
    Chill { radius: f32, amount: f32, secs: f32 },
    /// Make the player untouchable.
    Shield { secs: f32 },
    /// Temporary stat modifier on top of the final stats.
    Buff {
        stat: Stat,
        #[serde(default)]
        add: f32,
        #[serde(default)]
        mul: f32,
        secs: f32,
    },
    /// Scatter `count` mines around the event position.
    Mines { count: u32, radius: f32, damage: f32 },
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Rarity {
    #[default]
    Common,
    Rare,
    Epic,
}

impl Rarity {
    /// Offer weight after `picks` items taken this run. Rarer items grow
    /// more likely as the run goes on.
    pub fn weight(self, picks: u32) -> f32 {
        let p = picks as f32;
        match self {
            Rarity::Common => 10.0,
            Rarity::Rare => (4.0 + p * 0.2).min(8.0),
            Rarity::Epic => (1.0 + p * 0.12).min(4.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Weapon,
    Active,
    Passive,
}

/// Distinct weapon items a build can hold.
pub const MAX_WEAPONS: usize = 4;
/// Distinct active items a build can hold.
pub const MAX_ACTIVES: usize = 2;

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
    /// An extra weapon that attacks on its own.
    #[serde(default)]
    pub weapon: Option<WeaponDef>,
    #[serde(skip)]
    pub sprite_id: SpriteId,
}

impl ItemDef {
    pub fn kind(&self) -> Kind {
        if self.weapon.is_some() {
            Kind::Weapon
        } else if self.triggers.iter().any(|t| t.on == On::Active) {
            Kind::Active
        } else {
            Kind::Passive
        }
    }
}

fn default_stacks() -> u32 {
    5
}

/// Resolve sprites and cross-references for items and synergies, and check
/// rules the parser can't. Called once from `Content::load`.
pub fn resolve(items: &mut [ItemDef], synergies: &mut [SynergyDef]) -> Result<(), String> {
    for (i, it) in items.iter_mut().enumerate() {
        let owner = format!("item `{}`", it.id);
        for t in &mut it.triggers {
            t.owner = Some(i);
            if t.on == On::Active && t.cooldown <= 0.0 {
                return Err(format!("{owner}: Active triggers need a cooldown"));
            }
        }
        if let Some(w) = &mut it.weapon {
            w.sprite_id = sprite(&w.sprite, &owner)?;
        }
    }
    synergy::resolve(items, synergies)
}

fn sprite(name: &str, owner: &str) -> Result<SpriteId, String> {
    bank().id(name).ok_or_else(|| format!("{owner}: unknown sprite `{name}`"))
}

/// Items picked this run, in pick order.
#[derive(Clone, Debug, Default)]
pub struct Build {
    pub items: Vec<(usize, u32)>,
    /// Permanent upgrade modifiers from the save, applied like a passive.
    pub bonus: Vec<StatMod>,
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

    /// Items taken this run, counting stacks.
    pub fn picks(&self) -> u32 {
        self.items.iter().map(|(_, n)| n).sum()
    }

    /// Distinct items of a kind held.
    pub fn held(&self, content: &Content, kind: Kind) -> usize {
        self.items.iter().filter(|(i, _)| content.items[*i].kind() == kind).count()
    }

    /// Indices of synergies whose items are all held, in content order.
    pub fn synergies<'a>(&'a self, content: &'a Content) -> impl Iterator<Item = usize> + 'a {
        content.synergies.iter().enumerate().filter(|(_, s)| s.active(self)).map(|(k, _)| k)
    }

    /// Final stats: character base, then every modifier from the passive,
    /// upgrades, item stacks, and active synergies.
    pub fn stats(&self, ch: &CharacterDef, content: &Content) -> Stats {
        let mut base = Stats::defaults();
        for (s, v) in &ch.base {
            base.set(*s, *v);
        }
        self.with_mods(&base, ch, content)
    }

    /// The same modifiers on the default base stats. Item weapons and item
    /// effects use these, so an item works the same on every hero.
    pub fn item_stats(&self, ch: &CharacterDef, content: &Content) -> Stats {
        self.with_mods(&Stats::defaults(), ch, content)
    }

    fn with_mods(&self, base: &Stats, ch: &CharacterDef, content: &Content) -> Stats {
        let item_mods =
            self.items.iter().flat_map(|&(i, n)| (0..n).flat_map(move |_| content.items[i].stats.iter()));
        let synergy_mods = self.synergies(content).flat_map(|k| content.synergies[k].stats.iter());
        apply_mods(base, ch.stats.iter().chain(&self.bonus).chain(item_mods).chain(synergy_mods))
    }

    /// Every trigger instance. Each stack adds its triggers again; active
    /// synergies add theirs once.
    pub fn triggers(&self, ch: &CharacterDef, content: &Content) -> Vec<Trigger> {
        let mut out = ch.triggers.clone();
        for &(i, n) in &self.items {
            for _ in 0..n {
                out.extend(content.items[i].triggers.iter().cloned());
            }
        }
        for k in self.synergies(content) {
            out.extend(content.synergies[k].triggers.iter().cloned());
        }
        out
    }

    /// The synergy that taking `item` would complete, if any.
    pub fn completes(&self, content: &Content, item: usize) -> Option<usize> {
        if self.stacks(item) > 0 {
            return None;
        }
        content.synergies.iter().position(|s| {
            s.need_idx.contains(&item) && s.need_idx.iter().all(|&i| i == item || self.stacks(i) > 0)
        })
    }
}

/// Level-up weight for one item; zero means it can't be offered. Rarity sets
/// the base, owned items and synergy completions are favored, and weapon and
/// active slots are capped.
pub fn offer_weight(content: &Content, build: &Build, item: usize) -> f32 {
    let it = &content.items[item];
    let stacks = build.stacks(item);
    if stacks >= it.max_stacks {
        return 0.0;
    }
    let kind = it.kind();
    let slots = match kind {
        Kind::Weapon => MAX_WEAPONS,
        Kind::Active => MAX_ACTIVES,
        Kind::Passive => usize::MAX,
    };
    let held = build.held(content, kind);
    if stacks == 0 && held >= slots {
        return 0.0;
    }
    let mut w = it.rarity.weight(build.picks());
    if stacks > 0 {
        w *= 1.4;
    }
    if kind == Kind::Weapon && held == 0 {
        w *= 2.5;
    }
    if build.completes(content, item).is_some() {
        w *= 2.0;
    }
    w
}

/// Up to `n` distinct item choices, weighted by `offer_weight`, skipping locked items.
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
        .map(|(i, it)| if unlocked(it) { offer_weight(content, build, i) } else { 0.0 })
        .collect();
    let mut picks = Vec::with_capacity(n);
    // Until the build has an item weapon, one card is always a weapon.
    if build.held(content, Kind::Weapon) == 0 {
        let only: Vec<f32> = weights
            .iter()
            .enumerate()
            .map(|(i, &w)| if content.items[i].kind() == Kind::Weapon { w } else { 0.0 })
            .collect();
        if let Some(i) = rng.weighted(&only) {
            weights[i] = 0.0;
            picks.push(i);
        }
    }
    while picks.len() < n {
        let Some(i) = rng.weighted(&weights) else { break };
        weights[i] = 0.0;
        picks.push(i);
    }
    if picks.len() > 1 {
        let j = rng.below(picks.len());
        picks.swap(0, j);
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

    #[test]
    fn offers_respect_stacks_and_slots() {
        let content = crate::content::get();
        let weapons: Vec<usize> =
            (0..content.items.len()).filter(|&i| content.items[i].kind() == Kind::Weapon).collect();
        let mut build = Build::default();
        for &i in weapons.iter().take(MAX_WEAPONS) {
            build.add(i);
        }
        for &i in &weapons[MAX_WEAPONS..] {
            assert_eq!(offer_weight(content, &build, i), 0.0, "a fifth weapon was offered");
        }
        let held = weapons[0];
        assert!(offer_weight(content, &build, held) > 0.0, "a held weapon can still stack");
        for _ in 1..content.items[held].max_stacks {
            build.add(held);
        }
        assert_eq!(offer_weight(content, &build, held), 0.0, "a maxed item was offered");
    }
}
