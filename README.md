# 3D Tetris (Rust/Bevy)

A 3D implementation of Tetris using Rust and the Bevy game engine.

## Features
- **3D Gameplay**: Blocks fall in a 3D grid (8x8x15).
- **Advanced Shapes**: Includes standard 2D Tetris shapes and unique 3D pieces (Tripod, ScrewL, ScrewR).
- **Rotation**: Full 3D rotation (Q/E/R keys) with wall and floor kicks.
- **Visuals**: Top-down view, wireframe active block, solid landed blocks with height-based coloring for depth, and a guided 3D cage.
- **Layer Visualization**: Real-time 2D grid representations for each layer containing locked blocks, displayed on the right side of the screen for better spatial awareness.
- **Sound Effects**: Audio triggers for movement, rotation, drops, and clears.
- **Game States**: Pause (P), Game Over with Restart (R).

## Controls
- **WASD / Arrows**: Move the block on the X/Z plane.
- **Q / E / R**: Rotate active block around Y / X / Z axes.
- **Shift + Q/E/R**: Rotate in reverse direction.
- **Space**: Hard Drop.
- **P**: Pause Game.
- **R**: Restart (on Game Over).

## Running the Game
Prerequisites: Rust installed.

```bash
cargo run
```
Note: You must provide your own sound files in `assets/sounds/` (move.ogg, rotate.ogg, drop.ogg, clear.ogg) to hear audio.

## Development
- **Engine**: Bevy 0.17 (or latest stable)
- **Logic**: ECS architecture (Entities, Components, Systems).
