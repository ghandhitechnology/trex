# trex

A roguelite arena shooter that renders inside a terminal pane with kitty graphics. Built for playing in a tmux pane while agents work.

## Game

- Hybrid of Survivor.io and The Binding of Isaac. The player moves, weapons auto-fire at the nearest enemy, and Space dashes / triggers the active item.
- Endless escalating waves. Score is time survived; best time is saved per character.
- Level-ups offer a choice of items. Items stack and interact (synergies), so every run builds differently.
- Meta progression: runs earn currency that unlocks new items, enemies, and characters. Saved to disk.
- Multiple characters, each with a distinct starting weapon and passive.
- No sound.

## Controls

- WASD move, Space dash, P or Esc pause, Q quit (with confirm).
- Terminals inside tmux do not report key release. Movement is driven by key-repeat events with a short hold window and velocity smoothing so it feels continuous.
- The game pauses automatically on focus-out (tmux `focus-events on` sends `CSI O`).

## Rendering

- Targets Ghostty and Kitty, running inside tmux with `allow-passthrough on`.
- The game composes one low-res RGBA framebuffer (about 256x144, aspect matched to the pane) on the CPU every frame.
- The frame is transmitted with kitty graphics using a temp file or shared memory (`t=t` / `t=s`) so only a tiny escape crosses tmux. The same image id is overwritten each frame.
- The image is displayed with kitty Unicode placeholders (`U=1` virtual placement + `U+10EEEE` cells) so it stays attached to the pane.
- All escapes are wrapped in tmux DCS passthrough when `$TMUX` is set.
- A headless mode renders frames to PNG so the art can be checked without a terminal.

## Art

- Everything is drawn in code. No external assets.
- One fixed palette, 16x16 sprites, a consistent pixel-art style with outlines, and a built-in bitmap font.
- Juice matters: hit flash, particles, screen shake, damage numbers.

## Content

- Items, enemies, characters, and waves are data files embedded into the binary, so new content does not require engine changes.
- Code: Rust, single binary.
