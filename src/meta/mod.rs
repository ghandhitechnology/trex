//! Characters, meta progression, and the save file.

pub mod characters;
pub mod save;
pub mod sprites;

use save::Save;

/// Outcome of one run, fed into the save.
pub struct RunResult<'a> {
    pub character: &'a str,
    pub time: f32,
    pub kills: u32,
    pub level: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Reward {
    pub bones: u32,
    pub best: f32,
    pub new_best: bool,
}

/// Bones earned: one per 10 s survived, one per 25 kills, one per level.
pub fn bones_for(r: &RunResult) -> u32 {
    (r.time / 10.0) as u32 + r.kills / 25 + r.level.saturating_sub(1)
}

pub fn record_run(save: &mut Save, r: &RunResult) -> Reward {
    let bones = bones_for(r);
    save.bones += bones;
    save.runs += 1;
    let best = save.best.entry(r.character.to_string()).or_insert(0.0);
    let new_best = r.time > *best;
    if new_best {
        *best = r.time;
    }
    Reward { bones, best: *best, new_best }
}

/// True if a content entry with this id and unlock cost is available.
pub fn is_unlocked(save: &Save, id: &str, cost: u32) -> bool {
    cost == 0 || save.unlocked.contains(id)
}
