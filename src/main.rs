mod app;
mod bot;
mod content;
mod enemies;
mod engine;
mod game;
mod headless;
mod items;
mod meta;
mod render;
mod sim;
mod term;
mod ui;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
trex: a roguelite for a terminal pane

usage:
  trex                                  play
  trex --dump-frames DIR [--seconds N] [--seed S] [--every SECS] [--size WxH] [--hero ID] [--save fresh|unlocked|maxed]
                                        bot run, write PNG frames (4x)
  trex --sim [--runs N] [--seed S] [--max-secs N] [--hero ID] [--save fresh|unlocked|maxed] [--items]
                                        bot runs without rendering, print survival per hero
                                        (--items adds pick rates and survival deltas per item)
  trex --meta [--runs N] [--seed S] [--max-secs N]
                                        bot runs on one save, print unlock and feat pacing
  trex --stress [--minutes N] [--seed S] [--size WxH] [--hero ID]
                                        unkillable bot run, print entity counts and frame cost
  trex --sheet FILE                     write every sprite to one PNG

env:
  TREX_GFX=shm|file|direct   force the graphics transfer medium
  TREX_SCALE=N               force the integer upscale
  TREX_SAVE=PATH             save file location
  TREX_WARP=SECS             start the wave director that far into a run";

struct Args(Vec<String>);

impl Args {
    fn flag(&self, name: &str) -> bool {
        self.0.iter().any(|a| a == name)
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.0.iter().position(|a| a == name).and_then(|i| self.0.get(i + 1)).map(String::as_str)
    }

    fn parse<T: std::str::FromStr>(&self, name: &str, default: T) -> Result<T, String> {
        match self.value(name) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("bad value for {name}: {v}")),
        }
    }
}

fn size(s: &str) -> Result<(i32, i32), String> {
    let (w, h) = s.split_once('x').ok_or("size must look like 256x144")?;
    let (w, h) = (w.parse::<i32>().map_err(|e| e.to_string())?, h.parse::<i32>().map_err(|e| e.to_string())?);
    if !(64..=1024).contains(&w) || !(64..=1024).contains(&h) {
        return Err("size must be within 64..1024".into());
    }
    Ok((w, h))
}

/// `--hero ID`, checked against the content.
fn hero(args: &Args) -> Result<Option<&str>, String> {
    match args.value("--hero") {
        Some(id) if content::get().character(id).is_none() => Err(format!("unknown hero {id}")),
        v => Ok(v),
    }
}

fn run(args: &Args) -> Result<(), String> {
    if args.flag("--help") || args.flag("-h") {
        println!("{USAGE}");
        return Ok(());
    }
    if let Some(dir) = args.value("--dump-frames") {
        let opts = headless::DumpOptions {
            dir: &PathBuf::from(dir),
            seconds: args.parse("--seconds", 30.0)?,
            seed: args.parse("--seed", 1)?,
            every: args.parse("--every", 2.0)?,
            size: args.value("--size").map_or(Ok((256, 144)), size)?,
            hero: hero(args)?,
            start: args.value("--save").map_or(Ok(sim::Start::Fresh), sim::Start::parse)?,
        };
        return headless::dump_frames(&opts).map_err(|e| e.to_string());
    }
    if args.flag("--sim") {
        sim::sim(&sim::SimOptions {
            runs: args.parse("--runs", 20)?,
            seed: args.parse("--seed", 1)?,
            max_secs: args.parse("--max-secs", 3600.0)?,
            hero: hero(args)?,
            start: args.value("--save").map_or(Ok(sim::Start::Fresh), sim::Start::parse)?,
            items: args.flag("--items"),
        });
        return Ok(());
    }
    if args.flag("--meta") {
        sim::meta_sim(
            args.parse("--runs", 200)?,
            args.parse("--seed", 1)?,
            args.parse("--max-secs", 3600.0)?,
        );
        return Ok(());
    }
    if args.flag("--stress") {
        let size = args.value("--size").map_or(Ok((256, 144)), size)?;
        headless::stress(args.parse("--minutes", 60.0)?, args.parse("--seed", 1)?, size, hero(args)?);
        return Ok(());
    }
    if let Some(file) = args.value("--sheet") {
        return headless::sheet(&PathBuf::from(file)).map_err(|e| e.to_string());
    }
    if let Some(a) = args.0.first() {
        return Err(format!("unknown argument {a}\n\n{USAGE}"));
    }
    app::run().map_err(|e| e.to_string())
}

fn main() -> ExitCode {
    let args = Args(std::env::args().skip(1).collect());
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("trex: {e}");
            ExitCode::FAILURE
        }
    }
}
