# Rusty Tower Defense

A small tower defense game written in Rust with [macroquad](https://macroquad.rs).

Enemies march along a winding path toward your castle. Build and upgrade
towers on the grass to stop them. Survive all 30 waves to win.

## Running

```sh
cargo run --release
```

On Linux you may need the system libraries macroquad links against:

```sh
sudo apt install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

Run the tests with `cargo test`.

### Building a Windows .exe

On Windows, `cargo build --release` produces `target/release/tower_defense.exe`.
To cross-compile from Linux:

```sh
rustup target add x86_64-pc-windows-gnu
sudo apt install gcc-mingw-w64-x86-64
cargo build --release --target x86_64-pc-windows-gnu
```

The exe ends up in `target/x86_64-pc-windows-gnu/release/` and runs on its own,
no installer or extra files needed.

## How to play

You start with 150 gold and 20 lives. Killing enemies earns gold, and
clearing a wave pays a bonus. Enemies that reach the castle at the end of
the path damage it and cost you lives (tanks 2, bosses 10).

Towers can be placed anywhere on the grass, as long as they don't touch the
path or another tower.

| Input | Action |
| --- | --- |
| `1`-`5` or sidebar buttons | Pick a tower to place |
| Left click on grass | Place it (hold Shift to place several) |
| Right click / `Esc` / Cancel button | Stop placing, or deselect |
| Left click on a tower | Open its upgrade panel |
| `,` `.` `/` or the path cards | Buy the next upgrade on path 1, 2 or 3 |
| `Tab` or the Target button | Cycle targeting: First, Last, Strongest, Closest, Boss |
| `S` / `Delete` | Sell selected tower for 70% of what you spent |
| `Space` / `N` | Start the next wave |
| `A` | Toggle auto play (next wave starts on its own) |
| `F` | Cycle game speed (x1, x2, x3) |
| `P` | Pause |
| `M` | Switch map (before the first wave only) |
| `Esc` (nothing selected) or Settings button | Open the settings menu |
| `R` | Restart after game over or victory |

### Settings menu

- **Theme**: switch between day and night.
- **Tower guide**: every tower's stats and all 12 of its upgrades.
- **New game** / **New sandbox game**: restart in normal or sandbox mode.

### Sandbox mode

For testing builds. Towers and upgrades are free, lives never drop, and the
game never ends. The sidebar gets extra controls to spawn any enemy (Shift
for five at once), pick which wave comes next (this also sets how tough
spawned enemies are, up to wave 99) and clear the field.

### Towers

| Tower | Cost | Notes |
| --- | --- | --- |
| Arrow | 50 | Arrows fly through their target and keep going 2 tiles, hitting everything on the way |
| Cannon | 90 | Slow, splash damage |
| Frost | 70 | Pulses to freeze every enemy in range. Holds fire while everything in range is already frozen |
| Sniper | 120 | Global range, heavy damage, fires slowly |
| Farm | 150 | No attack. Pays 40 gold every time a wave is cleared |

Arrow, Cannon and Sniper have a targeting mode. Frost hits everything in
range, so it doesn't need one.

### Upgrades

Every tower has 3 upgrade paths with 4 tiers each. Like in BTD, you can only
upgrade 2 of the 3 paths on a tower, and only one of them past tier 2. Tower
labels such as `2-0-3` show the tiers bought on each path.

| Tower | Path 1 | Path 2 | Path 3 |
| --- | --- | --- | --- |
| Arrow | Sharp Arrows: damage | Rapid Fire: speed and multi-shot | Long Shot: range and pierce |
| Cannon | Big Bombs: damage, blast, stun | Rapid Reload: fire rate | Incendiary: burning |
| Frost | Deep Freeze: longer freezes, faster pulses | Frostbite: slows enemies after they thaw | Shatter: damage, hit enemies take extra damage |
| Sniper | Full Metal: damage, boss killer | Fast Firing: fire rate | Ricochet: bouncing shots, gold per wave |
| Farm | Crops: more gold per wave | Bank: interest on your gold each wave | Clinic: lives per wave |

### Enemies

- **Grunt**: the standard enemy.
- **Runner**: fast and fragile, arrives in bursts from wave 3.
- **Tank**: slow and tough, from wave 5.
- **Boss**: shows up every 5th wave, more each time. Freezes, slows and stuns only half affect it.

Enemy health grows every wave.

### Maps

- **Sketch** (default): a long path that loops over itself twice.
- **Zigzag**: a short path, so enemies reach the castle much sooner. Harder.

## Code layout

- `src/main.rs`: window setup and main loop
- `src/game.rs`: game state and simulation (spawning, targeting, projectiles, economy)
- `src/map.rs`: maps, path geometry and layout constants
- `src/tower.rs`: tower stats, upgrade paths and upgrade rules
- `src/enemy.rs`: enemy stats, movement and status effects
- `src/wave.rs`: wave composition and difficulty scaling
- `src/ui.rs`: input handling and button layout
- `src/render.rs`: all drawing
