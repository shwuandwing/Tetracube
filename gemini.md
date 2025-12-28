# Gemini Agent Context

## Project Overview
This is a 3D Tetris game written in Rust using Bevy.

## Key Files
- `src/main.rs`: Entry point. Contains App setup, ECS systems (Movement, Render, UI).
- `src/game.rs`: Core data structures (`GameGrid`, `Tetracube`, `TetracubeType`), and pure logic (rotation, collision).

## Architecture
- **Grid**: stored as `Vec<Option<TetracubeType>>` in `GameGrid` resource. 8x15x8 (w*h*d).
- **Coordinate System**: Y is Up (Height). X/Z are the horizontal plane.
- **Rendering**:
  - Active Block: `Gizmos` (Wireframe) with color-coded edges.
  - Landed Blocks: `Mesh3d` (Cuboids) using level-based (Y-coordinate) coloring for depth perception, with `Gizmos` indicators for piece type.
  - Camera: Top-down view shifted slightly left to accommodate UI.
  - UI:
    - **Layer Visualization**: Real-time 2D 8x8 grids for active layers (containing locked blocks). Uses a wrapping flexbox layout on the right side of the screen.
    - **Game Info**: Score, Level, and Next Piece preview.
  - Boundaries: `Gizmos` (3D Cage).
- **Gameplay**:
  - Includes all tetracube shapes (I, L, S, T, O, Tripod, ScrewL, ScrewR).
  - Rotation system with wall and floor kicks.
  - Level-based gravity speed progression.
  - **Grid Dirty Tracking**: `DirtyGrid(bool)` resource tracks when the grid needs redrawing for the 2D visualizer.
- **Refactoring**:
  - `Tetracube`: Shape and color logic moved from standalone functions to implementation methods.
  - `GameGrid`: Added helper methods `is_layer_empty` and `is_layer_full`.
- **State**: `GameState` enum (Playing, Paused, GameOver) manages transitions.
- **Audio**: Audio triggers for movement, rotation, clearing, and background music.

## Future Improvements
- Better visual effects (particles).
- Smooth interpolation for movement.
- High score persistence.
