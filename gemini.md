# Gemini Agent Context

## Project Overview
This is a 3D Tetris game written in Rust using Bevy.

## Key Files
- `src/main.rs`: Entry point. Contains App setup, ECS systems (Movement, Render, UI).
- `src/game.rs`: Core data structures (`GameGrid`, `Tetromino`, `TetrominoType`), and pure logic (rotation, collision).

## Architecture
- **Grid**: stored as `Vec<Option<Color>>` in `GameGrid` resource. 8x15x8 (w*h*d).
- **Coordinate System**: Y is Up (Height). X/Z are the horizontal plane.
- **Rendering**:
  - Active Block: `Gizmos` (Wireframe).
  - Landed Blocks: `Mesh3d` (Cuboids).
  - Boundaries: `Gizmos` (3D Cage).
- **State**: `GameState` enum manages transitions.
- **Audio**: Audio triggers for game events (requires assets in `assets/sounds/`).

## Future Improvements
- Better visual effects (particles).
- Smooth interpolation for movement.
- High score persistence.
