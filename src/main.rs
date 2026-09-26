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
mod term;
mod ui;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
trex: a roguelite for a terminal pane

usage:
  trex                                  play
  trex --dump-frames DIR [--seconds N] [--seed S] [--every SECS] [--size WxH]
                                        bot run, write PNG frames (4x)
  trex --sim [--runs N] [--seed S] [--max-secs N]
                                        bot runs without rendering, print stats
  trex --sheet FILE                     write every sprite to one PNG

env:
  TREX_GFX=shm|file|direct   force the graphics transfer medium
  TREX_SCALE=N               force the integer upscale
  TREX_SAVE=PATH             save file location";

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
        };
        return headless::dump_frames(&opts).map_err(|e| e.to_string());
    }
    if args.flag("--sim") {
        headless::sim(args.parse("--runs", 20)?, args.parse("--seed", 1)?, args.parse("--max-secs", 1800.0)?);
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
