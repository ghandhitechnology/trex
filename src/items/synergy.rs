//! Named synergies: bonuses that switch on while every listed item is held.
//! Besides plain stats and triggers, a synergy can upgrade an item weapon:
//! new sprite, more projectiles, extra on-hit actions.

use serde::Deserialize;

use super::{Action, Build, ItemDef, StatMod, Trigger, sprite};
use crate::render::sprite::SpriteId;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SynergyDef {
    pub id: String,
    pub name: String,
    pub desc: String,
    /// Item ids that must all be held.
    pub needs: Vec<String>,
    #[serde(default)]
    pub stats: Vec<StatMod>,
    #[serde(default)]
    pub triggers: Vec<Trigger>,
    #[serde(default)]
    pub upgrades: Vec<Upgrade>,
    #[serde(skip)]
    pub need_idx: Vec<usize>,
}

impl SynergyDef {
    pub fn active(&self, build: &Build) -> bool {
        self.need_idx.iter().all(|&i| build.stacks(i) > 0)
    }
}

/// Changes to one item weapon while the synergy is active.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Upgrade {
    /// Id of the item whose weapon changes.
    pub weapon: String,
    #[serde(default)]
    pub sprite: Option<String>,
    /// Extra projectiles, blades, mines, zaps, or beams.
    #[serde(default)]
    pub count: u32,
    /// Damage multiplier bonus: 0.5 is +50%.
    #[serde(default)]
    pub damage: f32,
    /// Radius multiplier bonus: 0.5 is +50%.
    #[serde(default)]
    pub area: f32,
    /// Attack rate multiplier bonus.
    #[serde(default)]
    pub rate: f32,
    #[serde(default)]
    pub pierce: u32,
    #[serde(default)]
    pub bounce: u32,
    /// Extra actions on every hit, at the hit enemy.
    #[serde(default)]
    pub on_hit: Vec<Action>,
    /// Replaces the weapon's end action.
    #[serde(default)]
    pub on_end: Option<Action>,
    #[serde(skip)]
    pub item: usize,
    #[serde(skip)]
    pub sprite_id: Option<SpriteId>,
}

pub fn resolve(items: &[ItemDef], synergies: &mut [SynergyDef]) -> Result<(), String> {
    let item = |id: &str, owner: &str| {
        items.iter().position(|it| it.id == id).ok_or_else(|| format!("{owner}: unknown item `{id}`"))
    };
    let mut seen = std::collections::HashSet::new();
    for s in synergies.iter_mut() {
        let owner = format!("synergy `{}`", s.id);
        if !seen.insert(s.id.clone()) {
            return Err(format!("duplicate synergy id `{}`", s.id));
        }
        if s.needs.len() < 2 {
            return Err(format!("{owner}: needs at least two items"));
        }
        s.need_idx = s.needs.iter().map(|id| item(id, &owner)).collect::<Result<_, _>>()?;
        for u in &mut s.upgrades {
            u.item = item(&u.weapon, &owner)?;
            if items[u.item].weapon.is_none() {
                return Err(format!("{owner}: item `{}` has no weapon", u.weapon));
            }
            if !s.need_idx.contains(&u.item) {
                return Err(format!("{owner}: upgrades `{}` without needing it", u.weapon));
            }
            u.sprite_id = u.sprite.as_deref().map(|n| sprite(n, &owner)).transpose()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::items::Build;

    #[test]
    fn synergies_switch_on_only_with_every_item() {
        let content = crate::content::get();
        for (k, s) in content.synergies.iter().enumerate() {
            let mut build = Build::default();
            for &i in &s.need_idx[..s.need_idx.len() - 1] {
                build.add(i);
            }
            assert!(!build.synergies(content).any(|a| a == k), "`{}` active too early", s.id);
            let last = *s.need_idx.last().unwrap();
            assert!(build.completes(content, last).is_some(), "`{}` not flagged as a combo", s.id);
            build.add(last);
            assert!(build.synergies(content).any(|a| a == k), "`{}` not active", s.id);
        }
    }
}
