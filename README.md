# Rusty Tower Defense

A small tower defense game written in Rust with [macroquad](https://macroquad.rs).
Capybaras defend against waves of cats.

Enemies march along a winding path toward your castle. Build and upgrade
towers on the grass to stop them. Survive all 30 waves to win.

## Versions

See [CHANGELOG.md](CHANGELOG.md). The current version is shown in the window title and the settings menu.

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

The game opens on the main menu. Pick a map (Sketch, Moon or Castle, marked
Easy, Medium and Hard) and a mode
(Normal or Sandbox), then press **Play**, or host or join a party.

You start with 300 gold and 100 lives. Killing enemies earns gold, and
clearing a wave pays a bonus. Enemies that reach the castle at the end of
the path damage it and cost you lives (tanks 2, bosses 10).

Towers can be placed anywhere on the grass, as long as they don't touch the
path or another tower.

| Input | Action |
| --- | --- |
| `1`-`6` or sidebar buttons | Pick a tower to place |
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
| `D` or Stats button | Damage meter: towers owned and damage dealt per tower type |
| `Esc` (nothing selected) or Settings button | Open the settings menu |
| `R` / Retry button | After losing: rewind to just before the lost wave and try it again |
| `Enter` / New game button | Start over after losing or winning |

### Levels and skill tree

Clearing waves earns XP (harder maps give more, and winning a whole game
gives a big bonus; sandbox gives none). Every level gives one skill point to
spend in the **Skills** screen on the main menu. Your profile is saved in
`%APPDATA%\RustyTowerDefense\profile.txt` on Windows (or
`~/.local/share/RustyTowerDefense/` elsewhere).

The tree has five branches. Each has one tier 1 skill (1 point) that unlocks
two tier 2 skills (2 points each):

| Branch | Tier 1 | Tier 2 |
| --- | --- | --- |
| Economy | Savings: +50 starting gold | Investor: +100 more gold / Bounty Hunter: +15% gold from cats |
| Defense | Thick Walls: +10 starting lives | Fortress: +20 more lives / Second Wind: +1 life per wave |
| Offense | Sharp Teeth: +5% damage | Fury: +10% more damage / Quick Paws: 8% faster attacks |
| Utility | Lookout: +5% range | Eagle Eyes: +10% more range / Deep Chill: freezes last 20% longer |
| Builder | Bargain: towers 5% cheaper | Haggler: upgrades 10% cheaper / Fair Trade: sell for 85% |

Points can be refunded at any time. In multiplayer, the host's skills apply.

### Settings menu

- **Theme**: switch between day and night.
- **Tower guide**: every tower's stats and all 12 of its upgrades.
- **Tower sounds** / **Game sounds**: volume sliders, 25% by default.
- **Restart** and **Main menu** (or **Leave party** in multiplayer).

### Multiplayer (co-op)

Up to 4 players share one game: the same gold, lives and towers, and anyone
can build, upgrade, sell or start waves.

- **Host party** starts a game with the map and mode you picked. The bottom
  left of the screen shows the address friends should join, e.g.
  `192.168.1.20:7777`.
- **Join party**: type the host's address in the box (the port is optional)
  and press Join.
- Players can join at any time, even mid-game. Menus don't pause the game
  in a party.
- It uses TCP port 7777. On a home network it works as is; Windows may ask
  to allow the game through the firewall the first time you host. Playing
  over the internet needs the host to forward port 7777 or use a LAN tool
  such as a VPN.

### Sandbox mode

For testing builds. Towers and upgrades are free, lives never drop, and waves
go on forever (the counter shows ∞). The sidebar gets extra controls to spawn any enemy (Shift
for five at once), pick which wave comes next (this also sets how tough
spawned enemies are, up to wave 99) and clear the field.

### Towers

| Tower | Cost | Notes |
| --- | --- | --- |
| Arrow (starter) | 50 | Arrows fly through their target and keep going 2 tiles, hitting everything on the way. Multi-shot upgrades fire a tight parallel volley |
| Mercenary (starter) | 60 | Gun for hire with three builds: rifle, dual pistols or light machine gun |
| Cannon | 90 | Slow, big splash damage |
| Frost | 70 | Pulses to freeze every enemy in range. A thawed enemy can't be refrozen for 1.5s. Holds fire while nothing in range can be frozen |
| Sniper | 120 | Global range, heavy damage, fires slowly |
| Farm | 150 | No attack. Pays 40 gold every time a wave is cleared |

Arrow, Mercenary, Cannon and Sniper have a targeting mode. Frost hits everything in
range, so it doesn't need one.

### Upgrades

Every tower is a capybara at its post. Each has 3 upgrade paths with 4 tiers.
Hover an upgrade card to see exactly which stats it changes. Towers also
change look as they level up: battlements at tier 2, a banner in their main
path's color at tier 3, and a crown and glow at tier 4. Like in BTD, you can only
upgrade 2 of the 3 paths on a tower, and only one of them past tier 2. Tower
labels such as `2-0-3` show the tiers bought on each path.

| Tower | Path 1 | Path 2 | Path 3 |
| --- | --- | --- | --- |
| Arrow | Sharp Arrows: damage | Rapid Fire: speed and multi-shot | Long Shot: range and pierce |
| Mercenary | Rifle: range, damage, armor-piercing rounds, boss killer | Pistols: dual wield, shoots several targets at once | Gunner: light machine gun, very fast fire, suppressing slow |
| Cannon | Big Bombs: damage, blast, stun | Rapid Reload: fire rate, up to 3x damage to tanks and bosses | Incendiary: burning |
| Frost | Deep Freeze: longer freezes, faster pulses | Frostbite: slows enemies after they thaw | Shatter: damage, hit enemies take extra damage |
| Sniper | Full Metal: damage, boss killer | Fast Firing: fire rate | Ricochet: bouncing shots, gold per wave |
| Farm | Crops: more gold per wave | Bank: interest on your gold each wave | Support: boosts attack speed, range and damage of towers nearby (frost-sized radius); buffed towers show a green arrow badge, and a tower you're placing shows it too when it would be buffed. Each tier also widens its reach |

### Enemies (cats)

- **House Cat**: the standard enemy.
- **Cheetah**: fast and fragile, arrives in bursts from wave 3.
- **Fluffy**: a big, slow, tough Persian, from wave 5.
- **Lion**: the boss, every 5th wave, more each time. Freezes, slows and stuns only half affect it.

Enemy health grows every wave.

### Maps

- **Sketch** (Easy, default): a long meadow path that loops over itself twice, with trees, rocks and a pond.
- **Moon** (Medium): rover tracks snaking between craters, with a lander and a lunar base to defend.
- **Castle** (Hard): a throne room. The track is a royal rug lined with torches, and
  the path is shorter, so enemies reach the throne sooner. Harder.

Trees, rocks, ponds, torches, pillars, suits of armor, craters and the lander block tower placement.

## Code layout

- `src/main.rs`: window setup, main menu flow and main loop
- `src/net.rs`: co-op multiplayer (host and client over TCP)
- `src/profile.rs`: XP, levels, the skill tree and the save file
- `src/audio.rs`: sound effects, synthesized at startup
- `src/game.rs`: game state and simulation (spawning, targeting, projectiles, economy)
- `src/map.rs`: maps, path geometry and layout constants
- `src/tower.rs`: tower stats, upgrade paths and upgrade rules
- `src/enemy.rs`: enemy stats, movement and status effects
- `src/wave.rs`: wave composition and difficulty scaling
- `src/ui.rs`: input handling and button layout
- `src/render.rs`: all drawing
