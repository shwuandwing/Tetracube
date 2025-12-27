# Gemini Agent Context

## Project Overview
This is a 3D Tetris game written in Rust using Bevy.

## Key Files
- `src/main.rs`: Entry point. Contains App setup, ECS systems (Movement, Render, UI).
- `src/game.rs`: Core data structures (`GameGrid`, `Tetromino`, `TetrominoType`), and pure logic (rotation, collision).

## Architecture
- **Grid**: stored as `Vec<Option<TetrominoType>>` in `GameGrid` resource. 8x15x8 (w*h*d).
- **Coordinate System**: Y is Up (Height). X/Z are the horizontal plane.
- **Rendering**:
  - Active Block: `Gizmos` (Wireframe) with color-coded edges.
  - Landed Blocks: `Mesh3d` (Cuboids) using level-based (Y-coordinate) coloring for depth perception, with `Gizmos` indicators for piece type.
  - Camera: Top-down view centered on the grid.
  - Boundaries: `Gizmos` (3D Cage).
- **Gameplay**:
  - Includes standard 2D shapes and complex 3D shapes (Tripod, ScrewL, ScrewR).
  - Rotation system with wall and floor kicks.
  - Level-based gravity speed progression.
- **State**: `GameState` enum (Playing, Paused, GameOver) manages transitions.
- **Audio**: Audio triggers for movement, rotation, clearing, and background music.

## Future Improvements
- Better visual effects (particles).
- Smooth interpolation for movement.
- High score persistence.
