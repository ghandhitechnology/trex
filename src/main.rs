mod app;
mod content;
mod enemies;
mod engine;
mod game;
mod items;
mod meta;
mod render;
mod term;
mod ui;

use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(e) = app::run() {
        eprintln!("trex: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
