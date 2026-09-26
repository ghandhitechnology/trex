# Content format

All game data lives in these RON files and is embedded into the binary with
`include_str!`. Loading is strict: unknown fields, unknown sprite names, and
unknown enemy ids fail at startup with the file and position. `cargo test`
runs the same load (`content::tests::embedded_content_loads`), so run it after
every data change.

| file | holds |
|------|-------|
| `items.ron` | level-up items |
| `synergies.ron` | named item combos |
| `enemies.ron` | enemy types |
| `characters.ron` | playable characters |
| `waves.ron` | the wave director |
| `meta.ron` | permanent upgrades and feats |

Fields marked *optional* can be left out. `items.ron` and `synergies.ron`
start with `#![enable(implicit_some)]`, so optional values are written
without `Some(...)`.

## Sprites

Content refers to sprites by name. Sprites are code, one module per area:

| area | module |
|------|--------|
| items, item projectiles | `src/items/sprites.rs` |
| enemies, enemy projectiles | `src/enemies/sprites.rs` |
| characters, weapon projectiles, meta icons | `src/meta/sprites.rs` |
| HUD and menus | `src/ui/sprites.rs` |
| gems, spawn warning | `src/render/sprites.rs` |

A sprite is a list of frames; a frame is rows of palette characters, `.` for
transparent. The character map is at the top of `src/render/palette.rs`.
`outline: true` adds a 1px ink outline (the frame grows by 1px per side), so
draw creatures in 14x14 to end up 16x16. Creatures face right; the renderer
flips them. Check art with `trex --sheet sheet.png`.

## Stats

Every tunable player number is a `Stat`. Final value:
`(base + sum of add) * (1 + sum of mul)`, clamped to the stat's limits.

| stat | meaning | default |
|------|---------|---------|
| `MaxHp` | half hearts | 6 |
| `Speed` | move speed, px/s | 68 |
| `Damage` | damage per shot | 6 |
| `FireRate` | volleys per second | 2.0 |
| `Shots` | projectiles per volley | 1 |
| `Spread` | degrees between projectiles | 12 |
| `ShotSpeed` | px/s | 170 |
| `ShotSize` | projectile size multiplier | 1 |
| `Range` | projectile travel, px | 140 |
| `Pierce` | extra enemies passed through | 0 |
| `Bounce` | redirects to a new enemy after a hit | 0 |
| `Homing` | turn rate toward enemies, rad/s | 0 |
| `Knockback` | push on hit, px/s | 60 |
| `Crit` | crit chance, 0..1 | 0.05 |
| `CritDamage` | crit multiplier | 2 |
| `Pickup` | gem magnet radius, px | 28 |
| `XpGain` | XP multiplier | 1 |
| `Regen` | HP per minute | 0 |
| `Dodge` | chance to ignore a hit, max 0.6 | 0 |
| `DashCooldown` | seconds | 1.25 |
| `Area` | radius multiplier for explosions, chains, shockwaves | 1 |
| `Duration` | multiplier for burn and slow | 1 |

A modifier: `(stat: Damage, add: 2.0)` or `(stat: FireRate, mul: 0.2)`.

## Triggers

A trigger runs an action when an event happens:

```ron
(on: Kill, chance: 0.3, cooldown: 0.0, action: Explode(radius: 18.0, damage: 0.8))
```

`chance` (optional, default 1) and `cooldown` (optional, seconds, default 0)
limit how often it fires. `from` (optional) limits `Hit`, `Crit` and `Kill`
to one damage source: `(on: Hit, from: Explode, action: Burn(...))`.

| `on` | fires when | position |
|------|------------|----------|
| `Hit` | player damage lands on an enemy (not burn ticks) | enemy |
| `Crit` | a hit crits | enemy |
| `Kill` | an enemy dies | enemy |
| `Dash` | the player dashes | player |
| `Hurt` | the player loses HP | player |
| `LevelUp` | the player takes a level-up item | player |
| `Timer` | every `cooldown` seconds | player |
| `Active` | the player presses Space; needs a `cooldown` | player |

Sources for `from`: `Main` (the character's weapon), `Gun`, `Orbit`, `Zap`,
`Beam`, `Mine`, `Aura`, `Meteor` (item weapons), and `Explode`, `Nova`,
`Chain`, `Volley` (actions).

| action | effect |
|--------|--------|
| `Burn(dps, secs)` | sets the hit enemy on fire; `dps` is a ratio of Damage; stacks add up to 2.5x Damage |
| `Slow(amount, secs)` | slows the hit enemy by `amount` (0..0.85) |
| `Explode(radius, damage)` | damages every enemy in the radius |
| `Nova(count, damage)` | fires `count` projectiles in a ring |
| `Chain(jumps, range, damage)` | lightning jumping between nearby enemies |
| `Volley(count, damage)` | fires `count` extra shots at the nearest enemies |
| `Shockwave(radius, force)` | pushes enemies away; negative `force` pulls |
| `Heal(amount)` | restores HP (integer, half hearts) |
| `Strike(count, radius, damage)` | drops meteors on random nearby enemies |
| `Chill(radius, amount, secs)` | slows every enemy in the radius |
| `Shield(secs)` | the player can't be hurt |
| `Buff(stat, add, mul, secs)` | temporary stat modifier on top of the final stats |
| `Mines(count, radius, damage)` | scatters mines around the event |

Rules that make synergies work without code:

- `damage` is always a ratio of the Damage stat, radii scale with `Area`,
  durations with `Duration`.
- Projectiles from `Nova` and `Volley` use the current projectile stats, so
  pierce, bounce, homing, and size apply to them too.
- Damage from actions fires `Hit` and `Kill` again, one generation deeper.
  Triggers fire for the weapon (generation 0) and first procs (generation 1),
  so chains stay bounded.
- Each item stack adds its modifiers and triggers again. Two stacks of an
  on-hit burn roll twice.

## items.ron

```ron
(
    id: "powder_keg",           // unique, used by saves and unlocks
    name: "Powder Keg",
    desc: "Kills may explode.", // one short line, shown on the card
    sprite: "powder_keg",
    rarity: Rare,               // optional: Common (default), Rare, Epic
    max_stacks: 5,              // optional, default 5
    unlock: 0,                  // optional: meta currency cost, 0 = unlocked
    stats: [],                  // optional: stat modifiers
    triggers: [                 // optional
        (on: Kill, chance: 0.3, action: Explode(radius: 18.0, damage: 0.8)),
    ],
    weapon: (...),              // optional, see below
)
```

An item is a **weapon** if it has `weapon`, an **active** if it has an
`Active` trigger (write "Space: ..." in its `desc`), and a **passive**
otherwise. A build holds at most 4 weapons and 2 actives; held ones can still
stack.

Level-ups offer three distinct items, skipping maxed and locked ones. Weights:

- rarity: Common 10, Rare 4 rising to 8, Epic 1 rising to 4 (+0.2 and +0.12
  per item taken this run)
- x1.4 for items already held, x2 for an item that completes a synergy
- x2.5 for weapons while none is held, and one card is always a weapon then

### Weapons

```ron
weapon: (
    kind: Orbit(radius: 22.0, spin: 3.6),
    sprite: "saw",      // projectile, blade, mine or meteor
    damage: 0.4,        // ratio of Damage per hit
    rate: 2.0,          // optional, attacks per second, default 1
    count: 2,           // optional, default 1
    on_hit: [],         // optional: actions at every enemy hit
    on_end: Nova(count: 6, damage: 0.5), // optional
)
```

| `kind` | does | `rate` means | `on_end` runs |
|--------|------|--------------|---------------|
| `Gun(arc, speed, range, pierce, bounce, motion)` | `count` shots in an `arc`-degree fan at the nearest enemy; 360 fires all around. `speed`/`range` multiply ShotSpeed/Range. `motion`: `Straight`, `Homing`, `Boomerang`. All fields optional | volleys/s | where a shot ends |
| `Orbit(radius, spin)` | `count` blades circle the player, `spin` rad/s | hits/s on one enemy | never |
| `Zap(jumps, range)` | `count` lightning bolts from the player that jump | zaps/s | at the last enemy |
| `Beam(length, width)` | `count` instant lines at the nearest enemies | beams/s | at the far end |
| `Mine(radius, secs)` | `count` mines at the player's feet; blow up on touch or after `secs` | drops/s | where it blew |
| `Aura(radius, slow)` | hurts (if `damage` > 0) and slows everything near the player | pulses/s | never |
| `Meteor(radius)` | `count` rocks fall on random nearby enemies | calls/s | where it lands |

Beams and auras take their colors from their sprite: the most used color is
the body, the second the highlight (see the swatches in `src/items/sprites.rs`).

Stats every weapon uses: Damage, FireRate (rate scales by FireRate over the
default 2.0), Shots (+count, except auras), Area (radii), Duration (mine
life), Crit. Guns also use every projectile stat. Item weapons and item
triggers read these stats with the stat defaults as their base instead of the
hero's `base`, so an item behaves the same on every hero; only modifiers
(items, upgrades, synergies, hero `stats`) change them. Weapon hits fire `Hit`,
`Crit` and `Kill` like the main weapon. Each stack past the first adds +10%
damage; stacks 2 and 4 add +1 count (auras grow 15% per stack instead).

## synergies.ron

A synergy switches on while every item in `needs` is held. It shows a
banner when it forms and a COMBO tag on the card that completes it.

```ron
(
    id: "fire_wheel",
    name: "Fire Wheel",
    desc: "Saws catch fire.",
    needs: ["saw_ring", "hot_lead"],
    stats: [],                  // optional, applied once
    triggers: [],               // optional, added once
    upgrades: [                 // optional: changes to a needed item's weapon
        (weapon: "saw_ring", sprite: "fire_saw", count: 1, on_hit: [Burn(dps: 0.8, secs: 3.0)]),
    ],
)
```

Upgrade fields, all optional except `weapon`: `sprite` (new look), `count`,
`damage` / `area` / `rate` (multiplier bonus, 0.5 is +50%), `pierce`,
`bounce`, `on_hit` (added), `on_end` (replaces).

## enemies.ron

```ron
(
    id: "brute",
    sprite: "brute",
    hp: 50.0,
    speed: 16.0,       // px/s
    damage: 2,         // optional, contact damage in half hearts, default 1
    radius: 8.0,       // optional, collision radius, default 6
    xp: 6,             // optional, integer, default 1; 5+ drops a big gem
    mass: 4.0,         // optional, knockback resistance, default 1
    cost: 6.0,         // optional, director credits, default 1
    behavior: Charge(range: 80.0, windup: 0.6, speed: 150.0, time: 0.45, cooldown: 1.6),
    shot: Some("spit"),                               // optional, projectile sprite, default "spit"
    minion: Some("mite"),                             // optional, for Summon and boss broods
    split: Some((enemy: "slimelet", count: 2)),       // optional, spawns on death
    shield: 30.0,      // optional, shield HP; soaks damage, regrows after 2.5s untouched
    death: Smoke,      // optional, death animation, default Pop
    name: "BONE COLOSSUS", // optional, shown on the boss bar
)
```

| behavior | does |
|----------|------|
| `Chase` (default) | walks at the player |
| `Weave(amp, freq)` | chases while swaying side to side |
| `Charge(range, windup, speed, time, cooldown)` | closes in, flashes, dashes in a line |
| `Shoot(range, cooldown, speed, pattern, windup)` | keeps distance, flashes for `windup`, fires `pattern` |
| `Orbit(radius, spin, dive)` | circles the player, dives through every `dive` seconds |
| `Burrow(under, up, shots)` | travels underground (untouchable), surfaces near the player with a ring of `shots` |
| `Blink(range, cooldown, windup, shots)` | vanishes, reappears `range` from the player, fires a fan |
| `Kamikaze(range, fuse, radius, damage)` | rushes in, lights a fuse, explodes; hurts enemies too |
| `Slam(range, windup, radius, damage, cooldown)` | winds up, pounds the ground around itself |
| `Summon(count, cooldown, range)` | keeps distance, calls in `count` of its `minion` |
| `Boss(kind)` | scripted boss: `Mire`, `Colossus`, `Wraith`, `Sandmaw`, `Eye` |

`pattern` is `Single` (default), `Spread(count, angle)` (degrees),
`Ring(count)`, or `Burst(count, gap)` (aimed shots `gap` seconds apart).
Every windup is telegraphed: the enemy flashes red and dangerous areas are
marked on the ground.

`death` is `Pop`, `Splat`, `Ash`, `Shatter`, `Smoke`, `Zap`, `Boom`, `Fade`,
or `Spores`. Particles take the sprite's colors.

Bosses rage at 55% and 25% HP: a roar, a ring of shots, a harder move list,
and shorter rests. A boss kill spills its XP as ten gems and heals 2 hearts.
Enemies with `hp` of 40 or more, and elites, show a health bar when damaged.

## characters.ron

```ron
(
    id: "rex",
    name: "Rex",
    desc: "Spits embers. Dashes knock enemies back.",
    sprite: "rex",                      // walk frames
    idle: "rex_idle",                   // optional: idle frames, else the first walk frame
    attack: "rex_attack",               // optional: pose played once per shot
    dash: "rex_dash",                   // optional: pose held through a dash
    muzzle: (6.0, -2.0),                // optional: where shots leave the sprite, px from center facing right
    legs: 11,                           // optional: art row where legs start; running keeps the stride's legs under the attack pose
    unlock: 0,                          // optional: bones to unlock, 0 = free
    weapon: (shot: "bolt", pattern: Aimed),
    base: { MaxHp: 8.0, FireRate: 2.0 }, // optional: overrides stat defaults for the hero and its weapon
    stats: [],                          // optional passive modifiers
    triggers: [                         // optional passive triggers
        (on: Dash, action: Shockwave(radius: 26.0, force: 170.0)),
    ],
)
```

`pattern` is `Aimed` (default: fan of `Shots` at the nearest enemy) or
`Radial` (`Shots` evenly around the player, no target needed).

Each hero has a walk sprite and an `_idle` sprite in `src/meta/sprites.rs`,
two frames each, plus `_attack` and `_dash` poses. Poses play once, frames
spread evenly over the pose. Check every pose with `trex --poses FILE` and
compare heroes with `trex --sim --runs 40`.

## waves.ron

```ron
(
    credits: 0.7,          // spawn credits per second at 0:00
    credits_per_min: 2.0,  // added to that each minute
    hp_growth: 0.27,       // enemy HP x1.27 per minute, compounding
    speed_per_min: 0.05,   // optional, enemy speed +5% per minute
    speed_max: 0.35,       // optional, up to +35%
    overtime: Some((from: 1320.0, every: 240.0)), // optional, see below
    max_alive: 320,        // cap on live + pending enemies
    stage_length: 150.0,   // seconds per stage
    loop_from: 1,          // optional, stage to loop back to after the last
    stages: [
        (name: "TAR PITS", pool: [("grub", 4.0), ("wisp", 2.0)]),   // (enemy id, weight)
        (name: "FROST CAVES", biome: Tarpit, ground: Some("kNb"), pool: [("frog", 2.0)]),
    ],
    events: [              // optional scripted groups
        (at: 60.0, every: 120.0, enemy: "wisp", count: 14, shape: Ring),
    ],
    elites: (from: 120.0, chance: 0.02, per_min: 0.012, max: 0.2, hp: 3.0, cost: 3.0),
    bosses: (first: 180.0, every: 180.0, order: ["mire_queen", "colossus"], calm: 0.3),
)
```

The director buys enemies from the current stage's pool with its credits.
Spawns land just off screen; ones that land on screen get a warning marker
first. Event `shape` is `Ring` (circle around the player) or `Cluster` (one
group off screen); `every` (optional) repeats the event, and counts grow 15%
per minute.

Stages change every `stage_length` seconds with a banner. `biome` (optional,
default `Tarpit`) picks the arena look: `Tarpit`, `Fernbog` or `Ashfall`.
`ground` (optional) is three palette characters (dark, mid, light) the floor
is recolored to on top of the biome. Both fade in over 3 seconds. After the
last stage the list loops from `loop_from`, and HP, credits, and elite odds
keep growing, so endless runs keep escalating. From `overtime.from` seconds
every hit on the player deals 1 more damage, plus 1 more each `every`
seconds, so no build lasts forever.

Elites roll per bought spawn (not events or summons) once the run passes
`from`: `chance` plus `per_min` each minute, capped at `max`. An elite has `hp`
times the health, costs `cost` times the credits, drops triple XP, and carries
one modifier shown as a colored rim: Swift (cyan, faster), Tough (gold, more
HP), Shielded (ice, shield bubble), Volatile (ember, bursts into shots on
death), Splitting (lime, splits in two).

Bosses arrive at `first` and then every `every` seconds, cycling through
`order`, with a banner 2.5 seconds ahead. While one is alive, regular spawn
credits run at `calm` times the normal rate. Boss HP grows with the square
root of the `hp_growth` multiplier, since each boss already sets its own HP.

`TREX_WARP=SECS` starts the director that far into a run (stage, bosses, HP,
credits) for checking late-game content.

## meta.ron

Runs pay bones: one per 20 s, one per 200 kills, one per two levels,
scaled by upgrades. Bones buy heroes and items with an `unlock` cost
and levels of permanent upgrades.

```ron
(
    upgrades: [
        (
            id: "sharp_teeth",
            name: "Sharp Teeth",
            desc: "+6% damage.",
            sprite: "up_damage",
            costs: [30, 60, 100],               // price per level; length = max level
            stats: [(stat: Damage, mul: 0.06)], // optional, added once per level
            bones: 0.0,                         // optional, extra bones per level (0.1 = +10%)
        ),
    ],
    feats: [
        (id: "horns_up", name: "Horns Up", goal: Survive(180.0), reward: Unlock("trike")),
    ],
)
```

Feats read lifetime records from the save, so a feat added later is granted
at startup if the records already meet it. The goal text is generated.

| goal | met when |
|------|----------|
| `Survive(secs)` | best time with any hero |
| `SurviveAs("id", secs)` | best time with that hero |
| `Kills(n)`, `Level(n)` | best single run |
| `Stacks(n)` | one item stacked `n` times in a run |
| `Items(n)` | `n` different items in a run |
| `TotalKills(n)`, `Runs(n)`, `Bones(n)` | lifetime totals (bones earned) |
| `Heroes(n)` | heroes unlocked, free ones included |
| `Upgrades(n)` | upgrade levels bought |

`reward` is `Bones(n)` or `Unlock("id")` (a hero or an item, free).

## Save file

`<data dir>/trex/save.ron` (or `$TREX_SAVE`), versioned (`meta::save::VERSION`).

- Every field has a default and unknown fields are ignored, so old files load
  and ids of removed content are kept as they are.
- The file holds only numbers, strings, and maps, so newer builds can add
  content types without breaking older readers.
- Older versions migrate in `save::parse`. A file from a newer version is
  read-only; a file that does not parse is moved to `save.ron.bad`.

## Checking changes

```sh
cargo test                                   # content loads and validates
trex --sheet /tmp/sheet.png                  # every sprite on one sheet
trex --dump-frames /tmp/frames --seconds 120 --seed 3   # also writes hub_*.png
trex --sim --runs 30 [--hero ID]             # survival per hero with a bot
trex --sim --runs 10 --save unlocked --items # pick rate and survival delta per item
trex --meta --runs 200                       # unlock and feat pacing over many runs
```
