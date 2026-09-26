//! Meta content from `content/meta.ron`: permanent upgrades and feats.

use serde::Deserialize;

use crate::content::Content;
use crate::game::clock_text;
use crate::items::StatMod;
use crate::render::sprite::{SpriteId, bank};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MetaDef {
    pub upgrades: Vec<UpgradeDef>,
    pub feats: Vec<FeatDef>,
}

/// A permanent upgrade bought with bones. Every level adds `stats` and `bones` again.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UpgradeDef {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub sprite: String,
    /// Price of each level; the length is the max level.
    pub costs: Vec<u32>,
    #[serde(default)]
    pub stats: Vec<StatMod>,
    /// Extra bones per run, as a fraction.
    #[serde(default)]
    pub bones: f32,
    #[serde(skip)]
    pub sprite_id: SpriteId,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FeatDef {
    pub id: String,
    pub name: String,
    pub goal: Goal,
    pub reward: FeatReward,
}

/// What a feat asks for. Every goal is read from the save, so feats added
/// later are granted at startup if the records already meet them.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum Goal {
    /// Survive this many seconds with any hero.
    Survive(f32),
    /// Survive this many seconds with one hero.
    SurviveAs(String, f32),
    /// Kills in one run.
    Kills(u32),
    /// Level reached in one run.
    Level(u32),
    /// Stacks of one item in one run.
    Stacks(u32),
    /// Different items in one run.
    Items(u32),
    /// Kills over all runs.
    TotalKills(u64),
    /// Runs played.
    Runs(u32),
    /// Bones earned over all runs.
    Bones(u64),
    /// Heroes unlocked, including free ones.
    Heroes(u32),
    /// Upgrade levels bought.
    Upgrades(u32),
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum FeatReward {
    Bones(u32),
    /// Unlocks a character or item for free.
    Unlock(String),
}

impl Goal {
    pub fn target(&self) -> f32 {
        match *self {
            Goal::Survive(s) | Goal::SurviveAs(_, s) => s,
            Goal::Kills(n) | Goal::Level(n) | Goal::Stacks(n) | Goal::Items(n) => n as f32,
            Goal::Runs(n) | Goal::Heroes(n) | Goal::Upgrades(n) => n as f32,
            Goal::TotalKills(n) | Goal::Bones(n) => n as f32,
        }
    }

    pub fn is_time(&self) -> bool {
        matches!(self, Goal::Survive(_) | Goal::SurviveAs(..))
    }

    /// One short line for the feats screen.
    pub fn text(&self, content: &Content) -> String {
        match self {
            Goal::Survive(s) => format!("Survive {}", clock_text(*s)),
            Goal::SurviveAs(id, s) => {
                let name = content.character(id).map_or(id.as_str(), |i| &content.characters[i].name);
                format!("Survive {} as {name}", clock_text(*s))
            }
            Goal::Kills(n) => format!("{n} kills in one run"),
            Goal::Level(n) => format!("Reach level {n}"),
            Goal::Stacks(n) => format!("Stack one item {n} times"),
            Goal::Items(n) => format!("Hold {n} different items"),
            Goal::TotalKills(n) => format!("{n} kills in total"),
            Goal::Runs(n) => format!("Play {n} runs"),
            Goal::Bones(n) => format!("Earn {n} bones"),
            Goal::Heroes(n) => format!("Unlock {n} heroes"),
            Goal::Upgrades(n) => format!("Buy {n} upgrade levels"),
        }
    }
}

/// What an unlock id refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unlockable {
    Hero(usize),
    Item(usize),
}

pub fn unlockable(content: &Content, id: &str) -> Option<Unlockable> {
    content
        .character(id)
        .map(Unlockable::Hero)
        .or_else(|| content.items.iter().position(|it| it.id == id).map(Unlockable::Item))
}

/// Resolve sprites and check every id the meta data refers to.
pub fn resolve(content: &mut Content) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for id in content.meta.upgrades.iter().map(|u| &u.id).chain(content.meta.feats.iter().map(|f| &f.id)) {
        if !seen.insert(id.clone()) {
            return Err(format!("meta: duplicate id `{id}`"));
        }
    }
    for u in &mut content.meta.upgrades {
        u.sprite_id = bank()
            .id(&u.sprite)
            .ok_or_else(|| format!("upgrade `{}`: unknown sprite `{}`", u.id, u.sprite))?;
        if u.costs.is_empty() {
            return Err(format!("upgrade `{}`: no costs", u.id));
        }
    }
    for f in &content.meta.feats {
        if let Goal::SurviveAs(id, _) = &f.goal
            && content.character(id).is_none()
        {
            return Err(format!("feat `{}`: unknown character `{id}`", f.id));
        }
        if let FeatReward::Unlock(id) = &f.reward
            && unlockable(content, id).is_none()
        {
            return Err(format!("feat `{}`: unknown unlock `{id}`", f.id));
        }
    }
    Ok(())
}
