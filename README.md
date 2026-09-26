# trex

A pixel-art roguelite arena shooter that runs inside a terminal pane. You
play a dinosaur, your weapons fire on their own, and you pick items on every
level-up until the waves overrun you. Runs pay out bones for permanent
upgrades, new items, and six more heroes.

Built to sit in a tmux pane next to your work. It pauses when the pane loses
focus.

## Install

```sh
brew install ghandhitechnology/trex/trex
trex
```

Or build it with Rust 1.85 or newer:
`cargo install --git https://github.com/ghandhitechnology/trex`.

The save lives in
`~/Library/Application Support/trex/save.ron` on macOS
(`~/.local/share/trex/` on Linux); `TREX_SAVE=PATH` moves it.

## Terminal

- **Ghostty** or **Kitty**. The game draws with the kitty graphics protocol
  and Unicode placeholders; other terminals are not supported.
- **tmux 3.3+** works. Add this to `~/.tmux.conf` and reload:

  ```tmux
  set -g allow-passthrough on
  set -g focus-events on
  ```

  `allow-passthrough` lets the image through tmux. `focus-events` lets the
  game pause when you switch panes.
- Any pane size works; the picture scales to fit. Around 100x30 cells or
  larger looks best.
- Over SSH the picture is softer, which keeps it light enough to stay smooth
  on a slow connection.

## Controls

| key | in a run | in menus |
|---|---|---|
| WASD or arrows | move | pick |
| Space or Enter | dash and fire active items | confirm |
| 1 to 9 | take a level-up card | |
| P or Esc | pause | back |
| Q | quit (Y confirms) | quit (Y confirms) |

## Development

```sh
trex --dump-frames DIR [--seconds N] [--seed S] [--every SECS] [--hero ID] [--save fresh|unlocked|maxed]
trex --clip DIR --from SECS --seconds N [--size WxH] [--seed S] [--hero ID] [--items ID,ID] [--levelup SECS] [--show run|title|paused|hub_*]
trex --sim --runs 30 [--hero ID] [--save fresh|unlocked|maxed] [--items]
trex --meta --runs 200
trex --stress [--minutes 60]
trex --sheet sheet.png
trex --poses poses.png
```

- `--dump-frames` plays a bot run and writes 4x PNGs of the title, every hub
  tab, gameplay, level-ups, pause, and death, so the art can be checked
  without a terminal.
- `--clip` fast-forwards an unkillable bot run and writes every frame at 1x
  (the trailer's footage, `trailer/tools/capture.sh`).
- `--poses` lays out every hero's idle, walk, attack, running attack, and
  dash frames, with the muzzle marked.
- `--sim` and `--items` print survival per hero and per item from bot runs.
  `--meta` shows when unlocks and feats land over many runs. `--stress`
  prints entity counts and frame cost per minute.
- `TREX_GFX=shm|file|direct` pins the image transfer, `TREX_SCALE=N` pins the
  upscale, `TREX_WARP=SECS` starts the waves that far in. `TREX_DEBUG=1`
  prints the image transfer, and over SSH the link's round trip, on exit.

Spec: `DESIGN.md`. Content format: `content/README.md`.
