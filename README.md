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

## How to play

You start with 150 gold and 20 lives. Killing enemies earns gold, and
clearing a wave pays a bonus. Each enemy that reaches the castle costs lives
(tanks cost 2, bosses cost 10).

| Input | Action |
| --- | --- |
| `1`-`4` or sidebar buttons | Pick a tower to build |
| Left click on grass | Build the picked tower (hold Shift to place several) |
| Left click on a tower | Select it |
| `U` | Upgrade selected tower (max level 3) |
| `S` / `Delete` | Sell selected tower for 70% of what you spent |
| `Space` / `N` | Start the next wave |
| `F` | Cycle game speed (x1, x2, x3) |
| `P` | Pause |
| Right click / `Esc` | Cancel selection |
| `R` | Restart after game over or victory |

### Towers

| Tower | Cost | Notes |
| --- | --- | --- |
| Arrow | 50 | Fast-firing, single target |
| Cannon | 90 | Slow, splash damage |
| Frost | 70 | Light damage, slows enemies (bosses resist half) |
| Sniper | 120 | Long range, heavy damage |

### Enemies

- **Grunt**: the standard enemy.
- **Runner**: fast and fragile, arrives in bursts from wave 3.
- **Tank**: slow and tough, from wave 5.
- **Boss**: shows up every 5th wave, more each time.

Enemy health grows every wave.

## Code layout

- `src/main.rs`: window setup and main loop
- `src/game.rs`: game state and simulation (spawning, targeting, projectiles, economy)
- `src/map.rs`: grid, path and layout constants
- `src/tower.rs`, `src/enemy.rs`: unit stats and behavior
- `src/wave.rs`: wave composition and difficulty scaling
- `src/ui.rs`: input handling and button layout
- `src/render.rs`: all drawing
