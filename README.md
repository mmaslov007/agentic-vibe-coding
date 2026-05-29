# Blox-Z

Blox-Z is a Rust + Bevy first-person zombie shooter prototype built from simple blocky geometry. Pick an arena, load in with a rifle and pistol, and clear wandering zombies before they hear you and swarm.

Everything in the prototype is made from procedural Bevy primitives, procedural textures, and generated pitch-based sound effects. No commercial game assets are used.

## Current Features

- First-person movement, mouse look, sprinting, and wall collision
- Rifle and pistol with ammo, reloads, recoil, tracers, and muzzle flashes
- In-game HUD for score, kills, active weapon, ammo, and reload state
- Zombies that wander, hear shots, chase, react to hits, and show health bars
- Procedural sounds for footsteps, guns, zombie idle noises, alerts, hits, and chase groans
- Main menu with map selection
- Escape pause menu with resume, main menu, and quit options
- Three playable maps:
  - `Desert Market`
  - `Greenwood`
  - `Night Quarter`

## Requirements

- Rust toolchain with Cargo
- A desktop environment that can open a Bevy window

On Windows, install Rust with:

```powershell
winget install Rustlang.Rustup
```

Restart the terminal after installing Rust so `cargo` is available.

## Run

From the project directory:

```powershell
cargo run
```

The first build may take several minutes because Bevy has to compile. Later runs should be faster.

If PowerShell cannot find `cargo`, use the Rustup-installed executable directly:

```powershell
& "$env:USERPROFILE\.cargo\bin\cargo.exe" run
```

## Controls

- `WASD`: Move
- `Mouse`: Look
- `Left Shift`: Sprint
- `Left Click`: Shoot or recapture cursor
- `1`: Equip rifle
- `2`: Equip pistol
- `R`: Reload
- `Escape`: Pause

## Verify

Run the test suite:

```powershell
cargo test
```

Check compilation without launching the game:

```powershell
cargo check
```

## Project Layout

- `src/main.rs`: App setup, window title, plugin registration
- `src/game_ui.rs`: Main menu, map selection, HUD, pause menu, score/ammo UI resources
- `src/player.rs`: First-person camera, movement, sprinting, footsteps
- `src/combat.rs`: Weapons, ammo state, reloads, hitscan shooting, tracers, scoring
- `src/zombies.rs`: Zombie spawning, AI, health bars, zombie sounds
- `src/map.rs`: Procedural maps, map lighting, textures, static colliders
- `src/audio_fx.rs`: Procedural sound handles and playback helper
- `src/collision.rs`: AABB movement and ray intersection helpers

## Roadmap Ideas

- Player health and zombie attacks
- Better authored zombie and weapon models
- Spatial audio and real sampled effects
- Game over and round restart flow
- Asset pipeline for maps beyond block geometry
