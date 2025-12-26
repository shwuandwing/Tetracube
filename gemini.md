# Gemini Agent Context

## Project Overview
This is a 3D Tetris game written in Rust using Bevy.

## Key Files
- `src/main.rs`: Entry point. Contains App setup, ECS systems (Movement, Render, UI).
- `src/game.rs`: Core data structures (`GameGrid`, `Tetromino`, `TetrominoType`).

## Architecture
- **Grid**: stored as `Vec<Option<Color>>` in `GameGrid` resource. 1D vector mapped to 3D (y * w * d + z * w + x).
- **Coordinate System**: Y is Up (Height). X/Z are the horizontal plane.
- **Rendering**:
  - Active Block: `Gizmos` (Wireframe).
  - Landed Blocks: `Mesh3d` (Cuboids).
- **State**: `GameState` enum manages transitions.

## Future Improvements
- Better visual effects (particles).
- Smooth interpolation for movement.
- Sound effects.
