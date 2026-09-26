//! Balance sims: bot runs without rendering. Survival per hero, per-item pick
//! rates and survival deltas, and meta progression over many runs. All of it
//! is deterministic per seed and runs on every core.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crate::bot::Bot;
use crate::content::{self, Content};
use crate::game::{Game, Scene, clock_text};
use crate::items::Kind;
use crate::meta::defs::{FeatReward, Goal};
use crate::meta::save::Save;
use crate::meta::{self, Buy};

/// Which meta state runs start from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// A new save: starting items only, no upgrades.
    Fresh,
    /// Every hero and item unlocked, no upgrades.
    Unlocked,
    /// Everything unlocked and every upgrade maxed.
    Maxed,
}

impl Start {
    pub fn parse(s: &str) -> Result<Start, String> {
        match s {
            "fresh" => Ok(Start::Fresh),
            "unlocked" => Ok(Start::Unlocked),
            "maxed" => Ok(Start::Maxed),
            _ => Err(format!("--save must be fresh, unlocked or maxed, not {s}")),
        }
    }

    pub fn save(self, hero: &str) -> Save {
        let c = content::get();
        let mut s = Save { character: hero.to_string(), ..Save::default() };
        s.unlocked.insert(hero.to_string());
        if self != Start::Fresh {
            s.unlocked.extend(c.characters.iter().map(|h| h.id.clone()));
            s.unlocked.extend(c.items.iter().map(|i| i.id.clone()));
        }
        if self == Start::Maxed {
            s.upgrades.extend(c.meta.upgrades.iter().map(|u| (u.id.clone(), u.costs.len() as u32)));
        }
        s
    }
}

#[derive(Clone, Debug, Default)]
pub struct RunStats {
    pub time: f32,
    pub level: u32,
    pub kills: u32,
    /// Items held at the end, with stacks.
    pub build: Vec<(usize, u32)>,
    /// Every level-up offer and the index of the item taken.
    pub offers: Vec<(Vec<usize>, usize)>,
}

/// Play one bot run with no rendering until death or `max_secs`. With
/// `force`, the bot takes that item whenever it is offered.
pub fn run(save: Save, seed: u64, max_secs: f32, force: Option<usize>) -> RunStats {
    let mut game = Game::new(save, None, seed, (256, 144), false);
    game.start_run();
    let mut bot = Bot::new(seed);
    bot.force = force;
    let mut offers = Vec::new();
    loop {
        let c = bot.controls(&game);
        if let (Scene::LevelUp(o), Some(k)) = (&game.scene, c.pick) {
            offers.push((o.items.clone(), o.items[k as usize - 1]));
        }
        game.update(&c);
        let w = game.world.as_ref().expect("run has a world");
        if matches!(game.scene, Scene::Dying(_) | Scene::Dead(_)) || w.time >= max_secs {
            return RunStats {
                time: w.time,
                level: w.player.level,
                kills: w.kills,
                build: w.build.items.clone(),
                offers,
            };
        }
    }
}

/// `f` over every job on all cores, results in job order.
fn par_map<T: Sync, R: Send>(jobs: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<R>>> = Mutex::new((0..jobs.len()).map(|_| None).collect());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|s| {
        for _ in 0..threads.min(jobs.len()) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(job) = jobs.get(i) else { break };
                    let r = f(job);
                    out.lock().expect("no panics while holding the lock")[i] = Some(r);
                }
            });
        }
    });
    out.into_inner().expect("threads joined").into_iter().map(|r| r.expect("every job ran")).collect()
}

pub struct SimOptions<'a> {
    pub runs: u32,
    pub seed: u64,
    pub max_secs: f32,
    /// One hero, or every hero when `None`.
    pub hero: Option<&'a str>,
    pub start: Start,
    /// Also print per-item pick rates and survival deltas.
    pub items: bool,
}

fn heroes(hero: Option<&str>) -> Vec<&'static str> {
    let c = content::get();
    c.characters.iter().map(|h| h.id.as_str()).filter(|id| hero.is_none_or(|h| h == *id)).collect()
}

fn mean(xs: impl Iterator<Item = f32>) -> f32 {
    let (sum, n) = xs.fold((0.0, 0), |(s, n), x| (s + x, n + 1));
    if n == 0 { 0.0 } else { sum / n as f32 }
}

/// Survival percentiles and means, one line.
fn summary_line(name: &str, stats: &[RunStats], max_secs: f32) -> String {
    let mut t: Vec<f32> = stats.iter().map(|s| s.time).collect();
    t.sort_by(f32::total_cmp);
    let q = |p: f32| clock_text(t[((t.len() - 1) as f32 * p).round() as usize]);
    let capped = t.iter().filter(|&&x| x >= max_secs).count();
    format!(
        "{name:<9}{:>6}{:>6}{:>6}{:>6}{:>6}{:>6}{:>6}{:>6.1}{:>7.0}{:>5}",
        clock_text(mean(t.iter().copied())),
        q(0.1),
        q(0.25),
        q(0.5),
        q(0.75),
        q(0.9),
        q(1.0),
        mean(stats.iter().map(|s| s.level as f32)),
        mean(stats.iter().map(|s| s.kills as f32)),
        capped,
    )
}

pub fn sim(o: &SimOptions) {
    let started = Instant::now();
    let heroes = heroes(o.hero);
    let jobs: Vec<(&str, u64)> =
        heroes.iter().flat_map(|&h| (0..o.runs as u64).map(move |r| (h, o.seed.wrapping_add(r)))).collect();
    let base = par_map(&jobs, |&(h, seed)| run(o.start.save(h), seed, o.max_secs, None));

    println!("save {:?}  {} runs per hero  cap {}", o.start, o.runs, clock_text(o.max_secs));
    println!("hero       mean   p10   p25   med   p75   p90   max   lvl  kills  cap");
    for (k, h) in heroes.iter().enumerate() {
        let chunk = &base[k * o.runs as usize..(k + 1) * o.runs as usize];
        println!("{}", summary_line(h, chunk, o.max_secs));
    }
    if heroes.len() > 1 {
        println!("{}", summary_line("all", &base, o.max_secs));
    }
    if o.items {
        item_report(o, &jobs, &base);
    }
    println!("elapsed {:.1}s", started.elapsed().as_secs_f32());
}

/// For every item: how often it was offered and taken by the bot, and the
/// mean survival when the bot always takes it, against the same seeds.
fn item_report(o: &SimOptions, jobs: &[(&str, u64)], base: &[RunStats]) {
    let c = content::get();
    let n = c.items.len();
    let (mut offered, mut taken) = (vec![0u32; n], vec![0u32; n]);
    for s in base {
        for (items, pick) in &s.offers {
            for &i in items {
                offered[i] += 1;
            }
            taken[*pick] += 1;
        }
    }
    let forced: Vec<(usize, &str, u64)> =
        (0..n).flat_map(|i| jobs.iter().map(move |&(h, seed)| (i, h, seed))).collect();
    let times = par_map(&forced, |&(i, h, seed)| run(o.start.save(h), seed, o.max_secs, Some(i)).time);
    let mut rows: Vec<(usize, f32)> =
        (0..n).map(|i| (i, mean(times[i * jobs.len()..(i + 1) * jobs.len()].iter().copied()))).collect();
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    let avg = mean(rows.iter().map(|r| r.1));
    println!(
        "\nforced: mean survival when the bot always takes the item; delta: against the average item ({})",
        clock_text(avg)
    );
    println!("item            kind     offered  pick%  forced   delta");
    for (i, t) in rows {
        let it = &c.items[i];
        let kind = match it.kind() {
            Kind::Weapon => "weapon",
            Kind::Active => "active",
            Kind::Passive => "passive",
        };
        let pick = if offered[i] > 0 { 100.0 * taken[i] as f32 / offered[i] as f32 } else { 0.0 };
        let delta = t - avg;
        let sign = if delta < 0.0 { "-" } else { "+" };
        println!(
            "{:<15} {kind:<8}{:>8}{:>6.0}%{:>8}   {sign}{}",
            it.id,
            offered[i],
            pick,
            clock_text(t),
            clock_text(delta.abs())
        );
    }
}

/// Everything the shop and hero list sell, with prices.
fn shop(c: &Content) -> Vec<Buy> {
    let heroes = (0..c.characters.len()).filter(|&i| c.characters[i].unlock > 0).map(Buy::Hero);
    heroes.chain(meta::shop_entries(c)).collect()
}

/// Bones still needed to own everything in the shop and hero list.
fn left_to_buy(save: &Save, c: &Content) -> u64 {
    shop(c)
        .into_iter()
        .map(|b| match b {
            Buy::Upgrade(i) => {
                let u = &c.meta.upgrades[i];
                u.costs.iter().skip(meta::upgrade_level(save, &u.id) as usize).map(|&p| u64::from(p)).sum()
            }
            _ => meta::price(save, c, b).map_or(0, u64::from),
        })
        .sum()
}

fn buy_name(c: &Content, b: Buy) -> String {
    match b {
        Buy::Hero(i) => format!("hero {}", c.characters[i].id),
        Buy::Item(i) => format!("item {}", c.items[i].id),
        Buy::Upgrade(i) => format!("upgrade {}", c.meta.upgrades[i].id),
    }
}

/// Play `runs` runs in a row on one save. After each run the bot buys the
/// cheapest thing it can afford until it can't, then plays its least played
/// hero. Prints when things unlock and how far off the lifetime feats are.
pub fn meta_sim(runs: u32, seed: u64, max_secs: f32) {
    let started = Instant::now();
    let c = content::get();
    let mut save = Save::default();
    let mut played = vec![0u32; c.characters.len()];
    let total = left_to_buy(&save, c);
    let mut milestones = [(0.1, None), (0.25, None), (0.5, None), (0.75, None), (1.0, None)];
    let mut events: Vec<String> = Vec::new();
    let mut kills = Vec::new();
    let mut bones = Vec::new();
    let mut times = Vec::new();
    for r in 1..=runs {
        let hero = (0..c.characters.len())
            .filter(|&i| meta::hero_unlocked(&save, c, i))
            .min_by_key(|&i| (played[i], std::cmp::Reverse(i)))
            .unwrap_or(0);
        played[hero] += 1;
        save.character = c.characters[hero].id.clone();
        let before = save.bones;
        let s = run(save.clone(), seed.wrapping_add(u64::from(r)), max_secs, None);
        let build = crate::items::Build { items: s.build.clone(), bonus: Vec::new() };
        let result = meta::RunResult {
            character: &c.characters[hero].id,
            time: s.time,
            kills: s.kills,
            level: s.level,
            build: &build,
        };
        let reward = meta::record_run(&mut save, c, &result);
        let earned = save.bones - before;
        kills.push(s.kills);
        bones.push(earned);
        times.push(s.time);
        if r <= 12 {
            events.push(format!(
                "{r:>5}  played {} {} +{earned} bones",
                c.characters[hero].id,
                clock_text(s.time)
            ));
        }
        for f in &reward.feats {
            events.push(format!("{r:>5}  feat {}", c.meta.feats[*f].id));
        }
        loop {
            let cheapest = shop(c)
                .into_iter()
                .filter_map(|b| meta::price(&save, c, b).map(|p| (p, b)))
                .min_by_key(|&(p, _)| p);
            let Some((p, b)) = cheapest.filter(|&(p, _)| p <= save.bones) else { break };
            if let Some(feats) = meta::purchase(&mut save, c, b) {
                events.push(format!("{r:>5}  {} ({p})", buy_name(c, b)));
                for f in feats {
                    events.push(format!("{r:>5}  feat {}", c.meta.feats[f].id));
                }
            }
        }
        let owned = total - left_to_buy(&save, c);
        for m in &mut milestones {
            if m.1.is_none() && owned as f32 >= total as f32 * m.0 {
                m.1 = Some(r);
            }
        }
    }
    println!("meta sim: {runs} runs, cheapest-first buying, least played hero");
    for e in &events {
        println!("{e}");
    }
    let at = |r: Option<u32>| r.map_or("never".to_string(), |r| format!("run {r}"));
    println!("\nshop and heroes worth {total} bones");
    for (frac, r) in milestones {
        println!("  {:>3.0}% owned   {}", frac * 100.0, at(r));
    }
    let per = |xs: &[u32], a: usize, b: usize| {
        let s = &xs[a.min(xs.len())..b.min(xs.len())];
        mean(s.iter().map(|&x| x as f32))
    };
    for (a, b) in [(0, 5), (5, 20), (20, 50), (50, 100), (100, runs as usize)] {
        if a < runs as usize {
            let t = mean(times[a..b.min(times.len())].iter().copied());
            println!(
                "  runs {:>3}-{:<4} bones/run {:>5.0}  kills/run {:>5.0}  mean {}",
                a + 1,
                b.min(runs as usize),
                per(&bones, a, b),
                per(&kills, a, b),
                clock_text(t)
            );
        }
    }
    let n = kills.len();
    let recent = |xs: &[u32]| per(xs, n.saturating_sub(50), n).max(1.0);
    let (kpr, bpr) = (recent(&kills), recent(&bones));
    println!("\nfeats done {}/{}", save.feats.len(), c.meta.feats.len());
    for f in c.meta.feats.iter().filter(|f| !save.feats.contains(&f.id)) {
        let now = meta::goal_value(&f.goal, &save, c);
        let left = f.goal.target() - now;
        let est = match f.goal {
            Goal::TotalKills(_) => format!("~{:.0} more runs", left / kpr),
            Goal::Bones(_) => format!("~{:.0} more runs", left / bpr),
            Goal::Runs(_) => format!("{left:.0} more runs"),
            _ => format!("{now:.0} of {:.0}", f.goal.target()),
        };
        let reward = match &f.reward {
            FeatReward::Bones(b) => format!("{b} bones"),
            FeatReward::Unlock(id) => id.clone(),
        };
        println!("  {:<14} {est:<18} -> {reward}", f.id);
    }
    println!("elapsed {:.1}s", started.elapsed().as_secs_f32());
}
