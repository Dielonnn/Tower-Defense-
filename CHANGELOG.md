# Changelog

Each release's number is set in `Cargo.toml` and shown in the window title and
the settings menu. The commit for each release is listed under its heading.

## v5.0

Commit: the one titled "v5.0: Minute Man, retry wave, easier start".

- New starter tower, Minute Man: a rifleman with Rifle, Pistols (dual wield) and Gunner (LMG) paths
- Arrow and Minute Man are the starter towers: first in the list, cheapest, tagged STARTER
- Start with 300 gold and 100 lives
- Enemies 5% slower, bosses 15% slower
- New games start at x1 speed with auto play off
- Retry the lost wave from the game over screen
- Castle map's rug is longer; the Sketch map's narrow left strip is now wide enough for towers
- Each Support path upgrade on the farm also widens its reach
- Sandbox wave counter shows the ∞ symbol

## v4.0

Commit: the one titled "v4.0: main menu, co-op multiplayer, audio, castle map".

- Main menu to pick the map and mode, host or join a party, open settings or quit
- Co-op multiplayer for up to 4 players over TCP (Host party / Join party)
- Sound effects for towers and the game, with Tower and Game volume sliders (25% default)
- Castle map (formerly Zigzag): stone hall with a rug for the track, torches,
  pillars, banners and a throne. Sketch map gets trees, rocks, flowers and pebbles
- More detailed towers, a ring color showing their highest tier, and a badge on buffed towers
- Frozen enemies can't be refrozen for 1.5s after thawing
- Arrow multi-shot upgrades fire parallel arrows in a tight volley
- Farm's Clinic path replaced by Support: boosts speed, range and damage of nearby towers
- Sandbox wave counter shows "Infinite" and waves are no longer capped at 99

## v3.0

Commit: the one titled "Label releases: v3.0".

- Enemies damage the castle at the end of the path (shake, flash, "-N")
- Sketch map straightened so every segment is horizontal or vertical
- Targeting modes for Arrow, Cannon and Sniper: First, Last, Strongest, Closest, Boss
- Sandbox mode: free towers and upgrades, no life loss, spawn any enemy, pick the wave, clear the field
- Settings menu with day/night themes, a tower guide and new game options
- Frost freezes instead of slowing; its Frostbite path adds a slow after thawing
- Arrow and Frost fire more slowly
- Farms earn more; the kill-bounty path is replaced by a Bank path that pays interest

## v2.0

Commit: `2cf5ffb`. Built before version labels, so the game itself doesn't show a number.


- Hand-drawn sketch map is the new default; the old zigzag map is still selectable
- Towers can be placed anywhere on the grass instead of on a grid
- Every tower has 3 upgrade paths with 4 tiers each (2 paths max, one past tier 2)
- New Farm tower that earns gold every wave
- Arrows pierce up to 2 tiles past their target
- Sniper has global range but fires 15% slower
- Frost hits everything in range and holds fire while everything is already slowed
- Auto play for waves and a visible cancel button while placing towers

## v1.0

Commit: `d0f855b`. Built before version labels, so the game itself doesn't show a number.


- First version: grid-based tower defense with Arrow, Cannon, Frost and Sniper
  towers, 3 upgrade levels, selling, 30 waves with runners, tanks and bosses,
  game speed and pause
- Release builds on Windows run without a console window
