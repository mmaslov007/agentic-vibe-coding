# Bevy FPS Dust Blockout

Small Rust + Bevy first-person shooter scaffold. The first playable goal is an empty, Dust2-inspired desert blockout with WASD walking, mouse look, sprinting, and simple wall collision.

This project does not use Counter-Strike assets. The map is an original graybox inspired by the high-level idea of long lanes, mid doors, tunnels, and two sites.

## Run

Install Rust first if `cargo` is not available:

```powershell
winget install Rustlang.Rustup
```

Then restart your terminal and run:

```powershell
cargo run
```

The first build can take a while because Bevy is being compiled.

## Verify

```powershell
cargo test
```

## Controls

- `WASD`: move
- `Mouse`: look
- `Left Shift`: sprint
- `Esc`: release cursor
- `Left click`: capture cursor again

## Project Shape

- `src/main.rs`: Bevy app setup and plugin registration
- `src/player.rs`: first-person camera controller and movement
- `src/map.rs`: map blockout and static collider registration
- `src/collision.rs`: reusable 2D horizontal AABB collision helpers

## Next Steps

- Replace blockout geometry with a real scene asset pipeline.
- Add jump/gravity and sloped ramps.
- Swap manual collision for a physics plugin if you need dynamic bodies.
- Add weapons, interactions, spawns, and game states.
