# trex

Roguelite arena shooter that renders inside a terminal pane with kitty
graphics. Spec: `DESIGN.md`. Data format: `content/README.md`.

```sh
cargo run --release                     # play (Ghostty or Kitty, tmux ok)
trex --dump-frames DIR --seconds 60 --seed 3 [--every 2] [--size 256x144]
trex --sim --runs 30 [--seed 1] [--max-secs 1800]
trex --sheet sheet.png
```

`--dump-frames` plays a bot run and writes 4x PNGs: `title.png`, a gameplay
frame every `--every` seconds, the first three level-up screens, `paused.png`
halfway, `dying.png` mid death transition, and `dead.png`. `--sim` runs bot games without rendering and prints
survival stats. Both are deterministic per seed.

Env: `TREX_GFX=shm|file|direct` pins the transfer medium, `TREX_SCALE=N`
pins the upscale, `TREX_SAVE=PATH` moves the save, `TREX_DEBUG=1` prints the
medium on exit. Save: `~/Library/Application Support/trex/save.ron` (macOS).

## Modules

Areas are split so they can change in parallel.

| area | files |
|------|-------|
| engine core | `engine/` (math, RNG, spatial grid), `game/` (scenes, world step order, player, shots, gems), `content.rs`, `app.rs` (loop), `term/` (terminal, kitty graphics, input) |
| items and effects | `items/mod.rs` (stats, triggers, items, level-up offers), `items/effects.rs` (trigger runner, actions), `items/sprites.rs`, `content/items.ron` |
| enemies and director | `enemies/mod.rs` (defs, per-tick update), `enemies/ai.rs` (behaviors), `enemies/director.rs` (waves), `enemies/sprites.rs`, `content/enemies.ron`, `content/waves.ron` |
| characters, meta, save | `meta/characters.rs`, `meta/mod.rs` (run rewards, unlocks), `meta/save.rs`, `meta/sprites.rs`, `content/characters.ron` |
| render, art, UI | `render/` (palette, canvas, sprites, font, camera, fx, arena floor, world drawing, PNG), `ui/` (HUD, screens, UI sprites) |
| headless | `headless.rs`, `bot.rs` |

The sim runs at a fixed 60 Hz (`engine::DT`). Gameplay uses `World::rng`
only; visual effects use their own RNG, so runs replay identically with or
without rendering.
