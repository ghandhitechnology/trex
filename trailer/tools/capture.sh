#!/usr/bin/env bash
# Captures the trailer's gameplay footage with the game's own renderer.
# Writes assets/footage/<id>/0000.png... at 1x native pixels and 30 fps, plus
# assets/footage/index.json. Every clip is a deterministic bot run
# (`trex --clip`), so the same seeds give the same frames.
set -euo pipefail

TRAILER="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$TRAILER/.." && pwd)"
OUT="$TRAILER/assets/footage"
TREX="$ROOT/target/release/trex"
FULL=320x180
# The tmux pane size: the smallest 16:9 size whose title screen fits cleanly.
PANE=224x126

(cd "$ROOT" && cargo build --release --quiet)
# The site's sliced sprites, shared with the trailer's drawn scenes.
rm -rf "$TRAILER/assets/sprites"
cp -R "$ROOT/site/assets/sprites" "$TRAILER/assets/sprites"
rm -rf "$OUT"
mkdir -p "$OUT"

INDEX=()

# clip ID SIZE SHOWS [trex --clip options...]
clip() {
  local id=$1 size=$2 shows=$3
  shift 3
  "$TREX" --clip "$OUT/$id" --size "$size" --fps 30 "$@" >/dev/null
  local frames
  frames=$(find "$OUT/$id" -name '*.png' | wc -l | tr -d ' ')
  INDEX+=("$(printf '  "%s": { "fps": 30, "frames": %d, "w": %d, "h": %d, "shows": "%s" }' \
    "$id" "$frames" "${size%x*}" "${size#*x}" "$shows")")
  echo "$id: $frames frames"
}

# Core loop and build.
clip early $FULL "Rex at 0:33, auto-fire hits, a dash shockwave at ~3.8s, gems" \
  --seed 10 --from 33 --seconds 5
clip levelup $FULL "level-up cards at 1.0s, a combo pick (Tar Pit) at 2.4s, play resumes" \
  --seed 10 --from 50 --seconds 4 --levelup 1
clip build $FULL "4:40, stacked build: saws, seekers, boomerangs, crowds and numbers" \
  --seed 7 --from 280 --seconds 5 --items magnet,magnet,saw_ring,seeker_pod,twin_barrel,bone_rang

# Synergies: the weapon stacked four times plus its partner.
clip syn-fire-wheel $FULL "Fire Wheel: burning saws orbit Rex" \
  --seed 2 --from 255 --seconds 2 --items saw_ring,saw_ring,saw_ring,saw_ring,hot_lead
clip syn-ball-lightning $FULL "Ball Lightning: boomerangs chain lightning" \
  --seed 4 --from 255 --seconds 2 --items bone_rang,bone_rang,bone_rang,bone_rang,static_coil
clip syn-extinction $FULL "Extinction: huge meteors land" \
  --seed 2 --from 255 --seconds 2 --items meteor_call,meteor_call,meteor_call,meteor_call,big_bones
clip syn-cryo-beam $FULL "Cryo Beam: ice beams through a crowd" \
  --seed 3 --from 255 --seconds 2 --items laser_eye,laser_eye,laser_eye,laser_eye,frost_halo

# Heroes mid-fight; rex, ptera and pachy dash inside the clip.
clip hero-rex $FULL "Rex, dash shockwave" --save unlocked --hero rex --seed 6 --from 96.9 --seconds 1
clip hero-ptera $FULL "Ptera, dash volley" --save unlocked --hero ptera --seed 3 --from 113.9 --seconds 1
clip hero-trike $FULL "Trike, piercing horns" --save unlocked --hero trike --seed 2 --from 100 --seconds 1
clip hero-raptor $FULL "Raptor, claw strikes" --save unlocked --hero raptor --seed 1 --from 100 --seconds 1
clip hero-spino $FULL "Spino, homing bubbles" --save unlocked --hero spino --seed 1 --from 100 --seconds 1
clip hero-stego $FULL "Stego, radial spikes" --save unlocked --hero stego --seed 4 --from 130 --seconds 1
clip hero-pachy $FULL "Pachy, exploding headbutt dash" --save unlocked --hero pachy --seed 4 --from 152.4 --seconds 1

# Bosses near the hero, boss bar on top.
clip boss-mire-queen $FULL "Mire Queen, 3:20" --save unlocked --seed 1 --from 200.6 --seconds 2
clip boss-colossus $FULL "Bone Colossus slams, 6:08" --save unlocked --seed 3 --from 367.9 --seconds 2
clip boss-wraith $FULL "Storm Wraith in a crowd, 9:28" --save unlocked --seed 2 --from 568.5 --seconds 2
clip boss-sandmaw $FULL "Sandmaw burrows next to the hero, 12:17" --save unlocked --seed 4 --from 737.4 --seconds 2
clip boss-hive-eye $FULL "Hive Eye, 15:31" --save unlocked --seed 4 --from 931.1 --seconds 2

# Late game.
clip chaos $FULL "14:12, hundreds of enemies, explosions, ~9800 kills" \
  --save unlocked --seed 1 --from 852 --seconds 6

# Meta progression stills.
clip hub-heroes $FULL "hub, heroes tab" --show hub_heroes --seconds 0
clip hub-locked $FULL "hub, a locked hero and its unlock goal" --show hub_locked --seconds 0
clip hub-shop $FULL "hub, upgrade shop" --show hub_shop --seconds 0
clip hub-items $FULL "hub, item unlocks" --show hub_items --seconds 0
clip hub-feats $FULL "hub, feats list" --show hub_feats --seconds 0

# The tmux pane: title, an early run, and the same run paused.
clip pane-title $PANE "title screen idling: logo drop, Rex" --show title --seconds 4
clip pane-run $PANE "early Rex run, 0:33-0:39" --seed 10 --from 33 --seconds 6
clip pane-paused $PANE "the same run paused at 0:39" --show paused --seed 10 --from 39 --seconds 0

{
  echo "{"
  for i in "${!INDEX[@]}"; do
    if ((i + 1 < ${#INDEX[@]})); then echo "${INDEX[$i]},"; else echo "${INDEX[$i]}"; fi
  done
  echo "}"
} >"$OUT/index.json"

total=$(find "$OUT" -name '*.png' | wc -l | tr -d ' ')
echo "wrote ${#INDEX[@]} clips, $total frames to $OUT"
