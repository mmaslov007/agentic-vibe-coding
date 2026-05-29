# Market Sweep

Market Sweep is a Rust + Bevy first-person shooter prototype. Pick a map, move through a compact arena, swap between an M16-inspired rifle and a USP-inspired pistol, and clear slow wandering zombies before they close in.

The project uses procedural blockout geometry, procedural textures, and generated pitch-based sound effects. It does not use Counter-Strike assets or other commercial game assets.

## Features

- First-person mouse look and WASD movement
- Rifle and pistol with ammo, reloads, tracers, and recoil
- Zombie AI with wandering, hearing, proximity aggro, chase behavior, hit reactions, health bars, and score rewards
- Menu flow with map selection and an Escape pause menu
- Three selectable maps:
  - `Desert Market`
  - `Greenwood`
  - `Night Quarter`
- Procedural audio for shooting, walking, zombie idle sounds, alerts, hit reactions, and chase groans

## Requirements

- Windows, macOS, or Linux
- Rust toolchain with Cargo

On Windows, install Rust with:

```powershell
winget install Rustlang.Rustup
```

Restart your terminal after installation so `cargo` is available on your `PATH`.

## Run The Game

From the project folder:

```powershell
cargo run
```

The first run can take several minutes because Bevy and its dependencies need to compile. Later runs should be much faster.

If `cargo` is not recognized in PowerShell but Rust is installed, try:

```powershell
& "$env:USERPROFILE\.cargo\bin\cargo.exe" run
```

## Controls

- `WASD`: Move
- `Mouse`: Look
- `Left Shift`: Sprint
- `Left Click`: Shoot or recapture the cursor
- `1`: Equip rifle
- `2`: Equip pistol
- `R`: Reload
- `Escape`: Pause menu

Pause menu options:

- `Return`: Resume the current match
- `Main Menu`: Return to map selection
- `Close Game`: Quit

## Verify

Run tests:

```powershell
cargo test
```

Check compilation without launching:

```powershell
cargo check
```

## Project Structure

- `src/main.rs`: Bevy app setup and plugin registration
- `src/game_ui.rs`: Main menu, map selection, HUD, pause menu, score state
- `src/player.rs`: First-person camera, movement, sprinting, footsteps
- `src/combat.rs`: Weapons, reloads, hitscan shots, tracers, scoring hooks
- `src/zombies.rs`: Zombie spawning, movement AI, health bars, zombie audio
- `src/map.rs`: Procedural map selection, geometry, textures, lighting, colliders
- `src/audio_fx.rs`: Procedural sound handles and playback helper
- `src/collision.rs`: AABB collision and ray intersection helpers

## Notes

This is still a scaffold, so maps are intentionally built from simple Bevy primitives. Good next upgrades would be a proper asset pipeline, spatial audio, player health/damage, and authored models for the environment and enemies.
