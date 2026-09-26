# Content format

All game data lives in these RON files and is embedded into the binary with
`include_str!`. Loading is strict: unknown fields, unknown sprite names, and
unknown enemy ids fail at startup with the file and position. `cargo test`
runs the same load (`content::tests::embedded_content_loads`), so run it after
every data change.

| file | holds |
|------|-------|
| `items.ron` | level-up items |
| `enemies.ron` | enemy types |
| `characters.ron` | playable characters |
| `waves.ron` | the wave director |

Fields marked *optional* can be left out.

## Sprites

Content refers to sprites by name. Sprites are code, one module per area:

| area | module |
|------|--------|
| items, item projectiles | `src/items/sprites.rs` |
| enemies, enemy projectiles | `src/enemies/sprites.rs` |
| characters, weapon projectiles | `src/meta/sprites.rs` |
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
| `DashCooldown` | seconds | 1.4 |
| `Area` | radius multiplier for explosions, chains, shockwaves | 1 |
| `Duration` | multiplier for burn and slow | 1 |

A modifier: `(stat: Damage, add: 2.0)` or `(stat: FireRate, mul: 0.2)`.

## Triggers

A trigger runs an action when an event happens:

```ron
(on: Kill, chance: 0.3, cooldown: 0.0, action: Explode(radius: 18.0, damage: 0.8))
```

`chance` (optional, default 1) and `cooldown` (optional, seconds, default 0)
limit how often it fires.

| `on` | fires when | position |
|------|------------|----------|
| `Hit` | player damage lands on an enemy (not burn ticks) | enemy |
| `Crit` | a hit crits | enemy |
| `Kill` | an enemy dies | enemy |
| `Dash` | the player dashes | player |
| `Hurt` | the player loses HP | player |
| `LevelUp` | the player takes a level-up item | player |
| `Timer` | every `cooldown` seconds | player |

| action | effect |
|--------|--------|
| `Burn(dps, secs)` | sets the hit enemy on fire; `dps` is a ratio of Damage; stacks add |
| `Slow(amount, secs)` | slows the hit enemy by `amount` (0..0.85) |
| `Explode(radius, damage)` | damages every enemy in the radius |
| `Nova(count, damage)` | fires `count` projectiles in a ring |
| `Chain(jumps, range, damage)` | lightning jumping between nearby enemies |
| `Volley(count, damage)` | fires `count` extra shots at the nearest enemies |
| `Shockwave(radius, force)` | pushes enemies away |
| `Heal(amount)` | restores HP (integer, half hearts) |

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
)
```

Level-ups offer three distinct items weighted by rarity (10 / 5 / 2),
skipping maxed and locked ones.

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
)
```

| behavior | does |
|----------|------|
| `Chase` (default) | walks at the player |
| `Weave(amp, freq)` | chases while swaying side to side |
| `Charge(range, windup, speed, time, cooldown)` | closes in, flashes, dashes in a line |
| `Shoot(range, cooldown, speed, sprite)` | keeps distance and fires `sprite` projectiles |

Enemies defined with `hp` of 40 or more show a health bar when damaged.

## characters.ron

```ron
(
    id: "rex",
    name: "Rex",
    desc: "Spits embers. Dashes knock enemies back.",
    sprite: "rex",
    unlock: 0,                          // optional
    weapon: (shot: "bolt", pattern: Aimed),
    base: { MaxHp: 6.0, FireRate: 2.0 }, // optional: overrides stat defaults
    stats: [],                          // optional passive modifiers
    triggers: [                         // optional passive triggers
        (on: Dash, action: Shockwave(radius: 26.0, force: 170.0)),
    ],
)
```

`pattern` is `Aimed` (default: fan of `Shots` at the nearest enemy) or
`Radial` (`Shots` evenly around the player, no target needed).

## waves.ron

```ron
(
    credits: 0.9,          // spawn credits per second at 0:00
    credits_per_min: 1.3,  // added to that each minute
    hp_growth: 0.25,       // enemy HP x1.25 per minute, compounding
    speed_per_min: 0.06,   // optional, enemy speed +6% per minute, max +50%
    max_alive: 320,        // cap on live + pending enemies
    phases: [              // the latest phase whose `at` has passed is active
        (at: 0.0, pool: [("grub", 1.0)]),         // (enemy id, weight)
    ],
    events: [              // optional scripted groups
        (at: 60.0, every: 120.0, enemy: "wisp", count: 14, shape: Ring),
    ],
)
```

The director buys enemies from the active pool with its credits. Spawns land
just off screen; ones that land on screen get a warning marker first. Event
`shape` is `Ring` (circle around the player) or `Cluster` (one group off
screen); `every` (optional) repeats the event, and counts grow 15% per minute.

## Checking changes

```sh
cargo test                                   # content loads and validates
trex --sheet /tmp/sheet.png                  # every sprite on one sheet
trex --dump-frames /tmp/frames --seconds 120 --seed 3
trex --sim --runs 30                         # survival stats with a bot
```
