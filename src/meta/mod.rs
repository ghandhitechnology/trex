//! Characters, meta progression, and the save file.
//!
//! Runs pay out bones. Bones buy heroes, items for the level-up pool, and
//! permanent upgrades. Feats read lifetime records from the save and grant
//! bones or free unlocks.

pub mod characters;
pub mod defs;
pub mod hub;
pub mod save;
pub mod screens;
pub mod sprites;

use std::collections::VecDeque;

use crate::content::Content;
use crate::items::{Build, StatMod};
use defs::{FeatReward, Goal, Unlockable};
use save::Save;

/// Outcome of one run, fed into the save.
pub struct RunResult<'a> {
    pub character: &'a str,
    pub time: f32,
    pub kills: u32,
    pub level: u32,
    pub build: &'a Build,
}

#[derive(Clone, Debug, Default)]
pub struct Reward {
    pub bones: u32,
    pub best: f32,
    pub new_best: bool,
    /// Feats completed by this run (indices into `content.meta.feats`).
    pub feats: Vec<usize>,
}

/// Bones before upgrades: one per 10 s survived, one per 25 kills, one per level.
pub fn base_bones(r: &RunResult) -> u32 {
    (r.time / 10.0) as u32 + r.kills / 25 + r.level.saturating_sub(1)
}

pub fn record_run(save: &mut Save, content: &Content, r: &RunResult) -> Reward {
    let bones = (base_bones(r) as f32 * (1.0 + bone_bonus(save, content))).round() as u32;
    save.bones = save.bones.saturating_add(bones);
    save.runs = save.runs.saturating_add(1);
    let rec = &mut save.records;
    rec.total_bones = rec.total_bones.saturating_add(u64::from(bones));
    rec.total_kills = rec.total_kills.saturating_add(u64::from(r.kills));
    rec.kills = rec.kills.max(r.kills);
    rec.level = rec.level.max(r.level);
    rec.items = rec.items.max(r.build.items.len() as u32);
    rec.stacks = rec.stacks.max(r.build.items.iter().map(|&(_, n)| n).max().unwrap_or(0));
    let best = save.best.entry(r.character.to_string()).or_insert(0.0);
    let new_best = r.time > *best;
    if new_best {
        *best = r.time;
    }
    let best = *best;
    let feats = check_feats(save, content);
    Reward { bones, best, new_best, feats }
}

/// True if a content entry with this id and unlock cost is available.
pub fn is_unlocked(save: &Save, id: &str, cost: u32) -> bool {
    cost == 0 || save.unlocked.contains(id)
}

pub fn hero_unlocked(save: &Save, content: &Content, i: usize) -> bool {
    let ch = &content.characters[i];
    is_unlocked(save, &ch.id, ch.unlock)
}

pub fn upgrade_level(save: &Save, id: &str) -> u32 {
    save.upgrades.get(id).copied().unwrap_or(0)
}

/// Stat modifiers from bought upgrades, one copy per level.
pub fn run_mods(save: &Save, content: &Content) -> Vec<StatMod> {
    let mut out = Vec::new();
    for u in &content.meta.upgrades {
        let level = upgrade_level(save, &u.id).min(u.costs.len() as u32);
        for _ in 0..level {
            out.extend(u.stats.iter().cloned());
        }
    }
    out
}

/// Extra bones per run from upgrades, as a fraction.
pub fn bone_bonus(save: &Save, content: &Content) -> f32 {
    content
        .meta
        .upgrades
        .iter()
        .map(|u| u.bones * upgrade_level(save, &u.id).min(u.costs.len() as u32) as f32)
        .sum()
}

/// Current value toward a goal, in the goal's units.
pub fn goal_value(goal: &Goal, save: &Save, content: &Content) -> f32 {
    let r = &save.records;
    match goal {
        Goal::Survive(_) => save.best.values().copied().fold(0.0, f32::max),
        Goal::SurviveAs(id, _) => save.best.get(id).copied().unwrap_or(0.0),
        Goal::Kills(_) => r.kills as f32,
        Goal::Level(_) => r.level as f32,
        Goal::Stacks(_) => r.stacks as f32,
        Goal::Items(_) => r.items as f32,
        Goal::TotalKills(_) => r.total_kills as f32,
        Goal::Runs(_) => save.runs as f32,
        Goal::Bones(_) => r.total_bones as f32,
        Goal::Heroes(_) => {
            (0..content.characters.len()).filter(|&i| hero_unlocked(save, content, i)).count() as f32
        }
        Goal::Upgrades(_) => content.meta.upgrades.iter().map(|u| upgrade_level(save, &u.id) as f32).sum(),
    }
}

/// Complete every feat whose goal is met and hand out rewards. Returns the new
/// ones in order. Loops because a reward can complete another feat.
pub fn check_feats(save: &mut Save, content: &Content) -> Vec<usize> {
    let mut done = Vec::new();
    loop {
        let before = done.len();
        for (i, f) in content.meta.feats.iter().enumerate() {
            if save.feats.contains(&f.id) || goal_value(&f.goal, save, content) < f.goal.target() {
                continue;
            }
            save.feats.insert(f.id.clone());
            match &f.reward {
                FeatReward::Bones(n) => {
                    save.bones = save.bones.saturating_add(*n);
                    save.records.total_bones = save.records.total_bones.saturating_add(u64::from(*n));
                }
                FeatReward::Unlock(id) => {
                    save.unlocked.insert(id.clone());
                }
            }
            done.push(i);
        }
        if done.len() == before {
            return done;
        }
    }
}

/// Something the shop or hero screen can sell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buy {
    Hero(usize),
    Item(usize),
    Upgrade(usize),
}

/// Price of the next purchase, or `None` when owned or maxed.
pub fn price(save: &Save, content: &Content, b: Buy) -> Option<u32> {
    match b {
        Buy::Hero(i) => (!hero_unlocked(save, content, i)).then_some(content.characters[i].unlock),
        Buy::Item(i) => {
            let it = &content.items[i];
            (!is_unlocked(save, &it.id, it.unlock)).then_some(it.unlock)
        }
        Buy::Upgrade(i) => {
            let u = &content.meta.upgrades[i];
            u.costs.get(upgrade_level(save, &u.id) as usize).copied()
        }
    }
}

/// Spend bones on `b`. Returns feats it completed, or `None` if it could not be bought.
pub fn purchase(save: &mut Save, content: &Content, b: Buy) -> Option<Vec<usize>> {
    let cost = price(save, content, b).filter(|&c| c <= save.bones)?;
    save.bones -= cost;
    match b {
        Buy::Hero(i) => {
            save.unlocked.insert(content.characters[i].id.clone());
        }
        Buy::Item(i) => {
            save.unlocked.insert(content.items[i].id.clone());
        }
        Buy::Upgrade(i) => *save.upgrades.entry(content.meta.upgrades[i].id.clone()).or_insert(0) += 1,
    }
    Some(check_feats(save, content))
}

/// The feat that unlocks `id`, if any.
pub fn feat_for_unlock(content: &Content, id: &str) -> Option<usize> {
    content.meta.feats.iter().position(|f| matches!(&f.reward, FeatReward::Unlock(u) if u == id))
}

/// Shop entries: upgrades first, then items that cost bones.
pub fn shop_entries(content: &Content) -> Vec<Buy> {
    let upgrades = (0..content.meta.upgrades.len()).map(Buy::Upgrade);
    let items = content.items.iter().enumerate().filter(|(_, it)| it.unlock > 0).map(|(i, _)| Buy::Item(i));
    upgrades.chain(items).collect()
}

pub fn unlock_name<'a>(content: &'a Content, id: &'a str) -> &'a str {
    match defs::unlockable(content, id) {
        Some(Unlockable::Hero(i)) => &content.characters[i].name,
        Some(Unlockable::Item(i)) => &content.items[i].name,
        None => id,
    }
}

/// A mid-progress save for headless previews of the meta screens.
pub fn demo_save() -> Save {
    let mut s = Save { bones: 185, runs: 9, character: "ptera".into(), ..Save::default() };
    s.best.extend([("rex".to_string(), 452.0), ("ptera".to_string(), 131.0)]);
    s.unlocked.extend(["ptera".to_string(), "spark_plug".to_string()]);
    s.upgrades.extend([
        ("sharp_teeth".to_string(), 2),
        ("tough_scales".to_string(), 1),
        ("fossil_hunter".to_string(), 1),
    ]);
    s.records =
        save::Records { kills: 380, level: 14, stacks: 3, items: 6, total_kills: 2400, total_bones: 900 };
    s
}

/// Feat banners waiting to be shown, one at a time.
#[derive(Default)]
pub struct Toasts {
    queue: VecDeque<usize>,
    age: f32,
}

pub const TOAST_TIME: f32 = 2.8;

impl Toasts {
    pub fn push(&mut self, feats: &[usize]) {
        self.queue.extend(feats);
    }

    pub fn update(&mut self, dt: f32) {
        if self.queue.is_empty() {
            return;
        }
        self.age += dt;
        if self.age >= TOAST_TIME {
            self.age = 0.0;
            self.queue.pop_front();
        }
    }

    /// The feat on screen and how long it has been shown.
    pub fn current(&self) -> Option<(usize, f32)> {
        self.queue.front().map(|&f| (f, self.age))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content;

    fn feat(c: &Content, id: &str) -> usize {
        c.meta.feats.iter().position(|f| f.id == id).unwrap()
    }

    #[test]
    fn purchases_and_upgrades() {
        let c = content::get();
        let mut s = Save { bones: 10_000, ..Save::default() };
        let up = Buy::Upgrade(0);
        let levels = c.meta.upgrades[0].costs.len();
        for _ in 0..levels {
            assert!(purchase(&mut s, c, up).is_some());
        }
        assert_eq!(price(&s, c, up), None);
        assert!(purchase(&mut s, c, up).is_none());
        let per_level = c.meta.upgrades[0].stats.len();
        assert_eq!(run_mods(&s, c).len(), per_level * levels);

        let locked = (0..c.characters.len()).find(|&i| !hero_unlocked(&s, c, i)).unwrap();
        let before = s.bones;
        purchase(&mut s, c, Buy::Hero(locked)).unwrap();
        assert!(hero_unlocked(&s, c, locked));
        assert_eq!(s.bones, before - c.characters[locked].unlock);

        let mut poor = Save::default();
        assert!(purchase(&mut poor, c, Buy::Hero(locked)).is_none());
    }

    #[test]
    fn feats_fire_once_and_grant_rewards() {
        let c = content::get();
        let mut s = Save::default();
        s.best.insert("rex".into(), 200.0);
        let got = check_feats(&mut s, c);
        assert!(got.contains(&feat(c, "hatchling")) && got.contains(&feat(c, "horns_up")));
        assert!(s.unlocked.contains("trike"));
        assert_eq!(s.bones, 25);
        assert!(check_feats(&mut s, c).is_empty());
    }

    #[test]
    fn run_updates_records_and_best() {
        let c = content::get();
        let mut s = Save::default();
        let mut build = Build::default();
        build.add(0);
        build.add(0);
        build.add(1);
        let r = RunResult { character: "rex", time: 95.0, kills: 60, level: 5, build: &build };
        let reward = record_run(&mut s, c, &r);
        assert!(reward.new_best);
        assert_eq!(reward.bones, 9 + 2 + 4);
        assert_eq!((s.records.stacks, s.records.items, s.records.kills), (2, 2, 60));
        assert!(reward.feats.contains(&feat(c, "hatchling")));
        let shorter = RunResult { time: 30.0, ..r };
        assert!(!record_run(&mut s, c, &shorter).new_best);
        assert_eq!(s.best["rex"], 95.0);
    }
}
