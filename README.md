# 3D Tetris (Rust/Bevy)

A 3D implementation of Tetris using Rust and the Bevy game engine.

## Features
- **3D Gameplay**: Blocks fall in a 3D grid (5x5x15).
- **Rotation**: Full 3D rotation (Q/E/R keys).
- **Visuals**: Wireframe active block, solid landed blocks with height-based coloring.
- **Game States**: Pause (P), Game Over with Restart (R).

## Controls
- **WASD / Arrows**: Move the block on the X/Z plane.
- **Q / E**: Rotate active block around Y/X axes.
- **R**: Rotate around Z axis (or Restart on Game Over).
- **Space**: Hard Drop.
- **P**: Pause Game.

## Running the Game
Prerequisites: Rust installed.

```bash
cargo run
```

## Development
- **Engine**: Bevy 0.17 (or latest stable)
- **Logic**: ECS architecture (Entities, Components, Systems).
