# Gemini Agent Context

## Project Overview
This is a 3D puzzle game involving tetracubes written in Rust using the Bevy game engine (v0.17).

## Key Files
- `src/main.rs`: Entry point. Contains App setup, ECS systems (Movement, Render, UI, States), and unit tests for systems.
- `src/game.rs`: Core data structures (`GameGrid`, `Tetracube`, `TetracubeType`), and pure logic (rotation, collision, hard drop calculation).
- `README.md`: User-facing documentation and controls.

## Architecture
- **Grid**: Stored as `Vec<Option<TetracubeType>>` in the `GameGrid` resource. Dimensions: 8x15x8 (W*H*D).
- **Coordinate System**: 
  - Y is Up (Height). X and Z define the horizontal plane.
  - Pivot-based movement and rotation.
- **Rendering**:
  - **Active Block**: Rendered with `Gizmos` (Wireframe) using piece-specific colors.
  - **Landed Blocks**: Rendered as `Mesh3d` (Cuboids). Colors are HSL-based rotating by Y-level for depth perception.
  - **Indicators**: `Gizmos` wireframes are rendered over landed blocks to indicate their original piece type.
  - **Camera**: Top-down view looking at `(center_x, 0, center_z)` from `y=25`.
  - **Boundaries**: A 3D cage rendered with `Gizmos`.
- **UI**:
  - **HUD**: Score, Level, and Next Piece preview. Managed via `GameUI` marker and `Visibility`.
  - **Layer Visualization**: Real-time 2D 8x8 grids for the bottom layer and any other layers containing blocks. Uses a wrapping flexbox layout on the right.
  - **Intro Screen**: Displays "T E T R A C U B E" title and start instructions.
- **Gameplay**:
  - **Shapes**: I, O, T, Z, L, and 3D shapes (Tripod, ScrewL, ScrewR).
  - **Movement**: WASD/Arrows. Timer-based for horizontal movement, separate timer for gravity.
  - **Rotation**: Q/E/R (Y/X/Z axes) with wall and floor kicks.
  - **Progression**: Level-based speed increase (score / 500).
- **State Management**:
  - `GameState` enum: `Intro` (default), `Playing`, `Paused`, `GameOver`.
  - `Intro` -> `Playing` via 'G' key.
  - `Playing` -> `Paused` via 'P' key.
  - `GameOver` -> `Playing` via 'R' key (resets grid).
- **Audio**: Sound effects for move, rotate, drop, and clear. Loops background music. Bundled assets live in `assets/sounds/` and can be regenerated with `python3 scripts/generate_audio.py`.
- **Testing**:
  - Pure logic tests in `src/game.rs`.
  - System tests in `src/main.rs` using a mock `App` with `MinimalPlugins` and `StatesPlugin`.

## Development Context
- **Grid Dirty Tracking**: `DirtyGrid(bool)` resource signals when 2D layer visualizations and landed block meshes need to be updated.
- **UI Visibility**: `GameUI` component is used to toggle the HUD visibility between `Intro` and `Playing` states.

## Future Improvements
- Visual polish (particles, animations).
- Smooth interpolation for movement.
- High score persistence.
- More complex 3D piece shapes.
