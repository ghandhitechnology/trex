# trex

Roguelite arena shooter that renders inside a terminal pane with kitty
graphics. Spec: `DESIGN.md`. Data format: `content/README.md`.

```sh
cargo run --release                     # play (Ghostty or Kitty, tmux ok)
trex --dump-frames DIR --seconds 60 --seed 3 [--every 2] [--size 256x144] [--hero ID]
trex --sim --runs 30 [--seed 1] [--max-secs 3600] [--hero ID] [--save fresh|unlocked|maxed] [--items]
trex --meta --runs 200 [--seed 1]
trex --stress [--minutes 60] [--seed 1] [--size 256x144] [--hero ID]
trex --sheet sheet.png
```

`--dump-frames` plays a bot run and writes 4x PNGs: `title.png`, five
`hub_*.png` menus over a mid-progress save, a gameplay frame every `--every`
seconds, the first three level-up screens, `paused.png` halfway, `dying.png`
mid death transition, and `dead.png`. `--sim` runs bot games without
rendering on every core and prints survival percentiles per hero from a
fresh, fully unlocked, or maxed save; `--items` adds each item's pick rate
and its survival delta when the bot always takes it. `--meta` plays runs in a
row on one save, buying the cheapest thing it can after each, and prints
when unlocks and feats land. `--stress` plays an unkillable bot run, renders
every tick like the live loop, and prints entity counts and frame cost per
minute. All of them are deterministic per seed.

The bot (`bot.rs`) plays like a decent player: it sees shots after a short
reaction delay, reads telegraphed charges, slams, fuses and burrows, scores a
ring of directions for danger over the next half second, and dashes out of
hits it can't walk away from. Balance targets: a fresh Rex run lasts about 6
minutes, a maxed save about 17, and overtime ends every run.

Env: `TREX_GFX=shm|file|direct` pins the transfer medium, `TREX_SCALE=N`
pins the upscale, `TREX_SAVE=PATH` moves the save, `TREX_WARP=SECS` starts
the wave director that far into a run, `TREX_DEBUG=1` prints the medium on
exit. Save: `~/Library/Application Support/trex/save.ron` (macOS).

## Modules

Areas are split so they can change in parallel.

| area | files |
|------|-------|
| engine core | `engine/` (math, RNG, spatial grid), `game/` (scenes, world step order, player, shots, gems), `content.rs`, `app.rs` (loop), `term/` (terminal, kitty graphics, input) |
| items and effects | `items/mod.rs` (stats, triggers, items, level-up offers), `items/weapons.rs` (item weapons, actives), `items/synergy.rs`, `items/effects.rs` (trigger runner, actions), `items/draw.rs`, `items/sprites.rs`, `content/items.ron`, `content/synergies.ron` |
| enemies and director | `enemies/mod.rs` (defs, per-tick update), `enemies/ai.rs` (behaviors), `enemies/boss.rs`, `enemies/director.rs` (stages, elites, bosses), `enemies/death.rs`, `enemies/draw.rs`, `enemies/sprites.rs`, `content/enemies.ron`, `content/waves.ron` |
| heroes, meta, save | `meta/characters.rs`, `meta/mod.rs` (run rewards, unlocks, feats), `meta/defs.rs`, `meta/hub.rs` and `meta/screens.rs` (hero select, shop, feats), `meta/save.rs`, `meta/sprites.rs`, `content/characters.ron`, `content/meta.ron` |
| render, art, UI | `render/` (palette, canvas, sprites, font, camera, fx, biome floors, lighting, world drawing, PNG), `ui/` (HUD, screens, UI sprites) |
| headless | `headless.rs`, `sim.rs`, `bot.rs` |

The sim runs at a fixed 60 Hz (`engine::DT`). Gameplay uses `World::rng`
only; visual effects use their own RNG, so runs replay identically with or
without rendering.
