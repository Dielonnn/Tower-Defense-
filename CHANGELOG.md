# Changelog

Each release's number is set in `Cargo.toml` and shown in the window title and
the settings menu. The commit for each release is listed under its heading.

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
