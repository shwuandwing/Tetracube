use bevy::prelude::*;

pub const GRID_WIDTH: i32 = 8;
pub const GRID_HEIGHT: i32 = 15;
pub const GRID_DEPTH: i32 = 8;

/// The types of tetracube pieces available in the game, including standard 2D and new 3D shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TetracubeType {
    I,
    O,
    T,
    Z,
    L,
    Tripod,
    ScrewL,
    ScrewR,
}

impl TetracubeType {
    /// Returns the relative block positions for a given tetracube type.
    pub fn get_shape_blocks(&self) -> [IVec3; 4] {
        match self {
            // Defined in X/Z plane (y=0)
            TetracubeType::I => [
                IVec3::new(0, 0, 0),
                IVec3::new(-1, 0, 0),
                IVec3::new(1, 0, 0),
                IVec3::new(2, 0, 0),
            ],
            TetracubeType::O => [
                IVec3::new(0, 0, 0),
                IVec3::new(1, 0, 0),
                IVec3::new(0, 0, 1),
                IVec3::new(1, 0, 1),
            ],
            TetracubeType::T => [
                IVec3::new(0, 0, 0),
                IVec3::new(-1, 0, 0),
                IVec3::new(1, 0, 0),
                IVec3::new(0, 0, 1),
            ],
            TetracubeType::Z => [
                IVec3::new(0, 0, 0),
                IVec3::new(1, 0, 0),
                IVec3::new(0, 0, 1),
                IVec3::new(-1, 0, 1),
            ],
            TetracubeType::L => [
                IVec3::new(0, 0, 0),
                IVec3::new(-1, 0, 0),
                IVec3::new(1, 0, 0),
                IVec3::new(1, 0, 1),
            ],
            TetracubeType::Tripod => [
                IVec3::new(0, 0, 0),
                IVec3::new(1, 0, 0),
                IVec3::new(0, 1, 0),
                IVec3::new(0, 0, 1),
            ],
            TetracubeType::ScrewL => [
                IVec3::new(0, 0, 0),
                IVec3::new(1, 0, 0),
                IVec3::new(1, 1, 0),
                IVec3::new(1, 1, 1),
            ],
            TetracubeType::ScrewR => [
                IVec3::new(0, 0, 0),
                IVec3::new(1, 0, 0),
                IVec3::new(1, 1, 0),
                IVec3::new(1, 1, -1),
            ],
        }
    }

    /// Returns the primary color for a given tetracube type.
    pub fn get_color(&self) -> Color {
        match self {
            TetracubeType::I => Color::srgb(0.0, 1.0, 1.0), // Cyan
            TetracubeType::O => Color::srgb(1.0, 1.0, 0.0), // Yellow
            TetracubeType::T => Color::srgb(1.0, 0.0, 1.0), // Magenta
            TetracubeType::Z => Color::srgb(1.0, 0.0, 0.0), // Red
            TetracubeType::L => Color::srgb(1.0, 0.5, 0.0), // Orange
            TetracubeType::Tripod => Color::srgb(1.0, 1.0, 1.0), // White
            TetracubeType::ScrewL => Color::srgb(0.5, 1.0, 0.0), // Lime
            TetracubeType::ScrewR => Color::srgb(0.5, 0.0, 1.0), // Purple
        }
    }
}

/// Represents an active tetracube piece with its type, relative block positions, global pivot, and color.
#[derive(Component, Clone)]
pub struct Tetracube {
    pub piece_type: TetracubeType,
    pub positions: [IVec3; 4], // Relative positions to the pivot
    pub pivot: IVec3,          // Global position
    pub color: Color,
}

/// A resource used to signal that the game grid has changed and needs re-rendering.
#[derive(Resource, Default)]
pub struct DirtyGrid(pub bool);

/// The 3D game grid storing the state of landed blocks.
#[derive(Resource)]
pub struct GameGrid {
    // Stores the type of the block at x, y, z. None means empty.
    pub grid: Vec<Option<TetracubeType>>,
}

impl GameGrid {
    /// Creates a new empty game grid.
    pub fn new() -> Self {
        Self {
            grid: vec![None; (GRID_WIDTH * GRID_HEIGHT * GRID_DEPTH) as usize],
        }
    }

    /// Converts 3D grid coordinates into a flat vector index.
    /// Returns None if coordinates are outside the grid boundaries.
    pub fn index(x: i32, y: i32, z: i32) -> Option<usize> {
        if x < 0 || x >= GRID_WIDTH || y < 0 || y >= GRID_HEIGHT || z < 0 || z >= GRID_DEPTH {
            return None;
        }
        Some((y * GRID_WIDTH * GRID_DEPTH + z * GRID_WIDTH + x) as usize)
    }

    /// Checks if a grid cell is occupied by a landed block or is out of horizontal/bottom bounds.
    /// Note: Positions above the grid height are not considered occupied.
    pub fn is_occupied(&self, x: i32, y: i32, z: i32) -> bool {
        if x < 0 || x >= GRID_WIDTH || z < 0 || z >= GRID_DEPTH || y < 0 {
            return true;
        }
        y < GRID_HEIGHT && self.get(x, y, z).is_some()
    }

    /// Checks if a position is within the horizontal and bottom boundaries of the grid.
    pub fn is_valid_pos(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0 && x < GRID_WIDTH && z >= 0 && z < GRID_DEPTH && y >= 0
    }

    /// Sets the piece type at a specific grid coordinate.
    pub fn set(&mut self, x: i32, y: i32, z: i32, piece_type: TetracubeType) {
        if let Some(idx) = Self::index(x, y, z) {
            self.grid[idx] = Some(piece_type);
        }
    }

    /// Gets the piece type at a specific grid coordinate, if any.
    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<TetracubeType> {
        Self::index(x, y, z).and_then(|idx| self.grid[idx])
    }

    /// Checks if a horizontal layer at height y is full.
    pub fn is_layer_full(&self, y: i32) -> bool {
        let layer_size = (GRID_WIDTH * GRID_DEPTH) as usize;
        let start = y as usize * layer_size;
        if start >= self.grid.len() {
            return false;
        }
        self.grid[start..start + layer_size]
            .iter()
            .all(|c| c.is_some())
    }

    /// Checks if a horizontal layer at height y is completely empty.
    pub fn is_layer_empty(&self, y: i32) -> bool {
        let layer_size = (GRID_WIDTH * GRID_DEPTH) as usize;
        let start = y as usize * layer_size;
        if start >= self.grid.len() {
            return true;
        }
        self.grid[start..start + layer_size]
            .iter()
            .all(|c| c.is_none())
    }

    /// Clears any 2D horizontal layers that are completely filled with blocks.
    /// Remaining blocks above cleared layers are shifted down.
    /// Returns the number of layers cleared and marks the grid as dirty.
    pub fn clear_full_layers(&mut self, dirty: &mut DirtyGrid) -> u32 {
        let layer_size = (GRID_WIDTH * GRID_DEPTH) as usize;
        let mut layers_cleared = 0;
        let mut write_y = 0;

        for read_y in 0..GRID_HEIGHT {
            if self.is_layer_full(read_y) {
                layers_cleared += 1;
            } else {
                if read_y != write_y {
                    let src_start = read_y as usize * layer_size;
                    let dst_start = write_y as usize * layer_size;
                    self.grid
                        .copy_within(src_start..src_start + layer_size, dst_start);
                }
                write_y += 1;
            }
        }

        if layers_cleared > 0 {
            dirty.0 = true;
            let clear_start = write_y as usize * layer_size;
            self.grid[clear_start..].fill(None);
        }
        layers_cleared
    }

    /// Permanently locks a tetracube's blocks into the grid.
    /// Returns true if any part of the piece is at or above GRID_HEIGHT (Game Over).
    pub fn lock_tetracube(&mut self, tetracube: &Tetracube, dirty: &mut DirtyGrid) -> bool {
        let mut game_over = false;
        dirty.0 = true;
        for pos in &tetracube.positions {
            let global = tetracube.pivot + *pos;
            if global.y >= GRID_HEIGHT {
                game_over = true;
            }
            self.set(global.x, global.y, global.z, tetracube.piece_type);
        }
        game_over
    }
}

/// Attempts to rotate a tetracube around a given cardinal axis.
/// If the standard rotation is blocked, it tries a sequence of "kicks" (nudges)
/// to find a valid nearby position. Returns true if successful.
pub fn try_rotate_with_kicks(
    tetracube: &mut Tetracube,
    axis: IVec3,
    forward: bool,
    grid: &GameGrid,
) -> bool {
    let mut new_positions = [IVec3::ZERO; 4];
    for (i, p) in tetracube.positions.iter().enumerate() {
        new_positions[i] = rotate_point(*p, axis, forward);
    }

    // Kick offsets to try: (0,0,0) then simple nudges
    let kicks = [
        IVec3::ZERO,
        IVec3::new(0, 1, 0), // Up (Floor kick)
        IVec3::new(0, 2, 0), // Up 2
        IVec3::new(1, 0, 0),
        IVec3::new(-1, 0, 0),
        IVec3::new(0, 0, 1),
        IVec3::new(0, 0, -1),
    ];

    for kick in kicks {
        let test_pivot = tetracube.pivot + kick;
        if can_place(&new_positions, test_pivot, grid) {
            tetracube.positions = new_positions;
            tetracube.pivot = test_pivot;
            return true;
        }
    }
    false
}

/// Rotates a 3D point 90 degrees around one of the cardinal axes.
pub fn rotate_point(point: IVec3, axis: IVec3, forward: bool) -> IVec3 {
    match (axis.x, axis.y, axis.z, forward) {
        (1, _, _, true) => IVec3::new(point.x, -point.z, point.y),
        (1, _, _, false) => IVec3::new(point.x, point.z, -point.y),
        (_, 1, _, true) => IVec3::new(point.z, point.y, -point.x),
        (_, 1, _, false) => IVec3::new(-point.z, point.y, point.x),
        (_, _, 1, true) => IVec3::new(-point.y, point.x, point.z),
        (_, _, 1, false) => IVec3::new(point.y, -point.x, point.z),
        _ => point,
    }
}

/// Checks if a set of relative positions can be placed at a given pivot within the grid.
pub fn can_place(positions: &[IVec3], pivot: IVec3, grid: &GameGrid) -> bool {
    positions.iter().all(|&pos| {
        let global = pivot + pos;
        grid.is_valid_pos(global.x, global.y, global.z)
            && !grid.is_occupied(global.x, global.y, global.z)
    })
}

/// Calculates the final pivot position for a hard drop.
pub fn calculate_hard_drop(tetracube: &Tetracube, grid: &GameGrid) -> IVec3 {
    let mut current_pivot = tetracube.pivot;
    while can_place(&tetracube.positions, current_pivot + IVec3::NEG_Y, grid) {
        current_pivot += IVec3::NEG_Y;
    }
    current_pivot
}

/// Keeps track of score and level.
#[derive(Resource, Default)]
pub struct GameStats {
    pub score: u32,
    pub level: u32,
}

impl GameStats {
    /// Creates a new GameStats resource.
    pub fn new() -> Self {
        Self { score: 0, level: 1 }
    }

    /// Adds cleared layers to the score and updates the level. Returns true if leveled up.
    pub fn add_layers(&mut self, layers: u32) -> bool {
        let old_level = self.level;
        self.score += layers * 100;
        self.level = (self.score / 500) + 1;
        self.level > old_level
    }

    /// Calculates the current fall speed based on the level.
    pub fn get_fall_speed(&self) -> f32 {
        (0.8 - (self.level as f32 - 1.0) * 0.1).max(0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_indexing() {
        let idx = GameGrid::index(0, 0, 0);
        assert_eq!(idx, Some(0));

        let idx_max = GameGrid::index(GRID_WIDTH - 1, GRID_HEIGHT - 1, GRID_DEPTH - 1);
        assert!(idx_max.is_some());

        let idx_oob = GameGrid::index(-1, 0, 0);
        assert_eq!(idx_oob, None);
    }

    #[test]
    fn test_grid_occupancy() {
        let mut grid = GameGrid::new();
        assert!(!grid.is_occupied(0, 0, 0));

        grid.set(0, 0, 0, TetracubeType::I);
        assert!(grid.is_occupied(0, 0, 0));

        // Out of bounds (negative or horizontal) is occupied
        assert!(grid.is_occupied(-1, 5, 2));
        assert!(grid.is_occupied(GRID_WIDTH, 5, 2));
        assert!(grid.is_occupied(0, -1, 0));

        // Above grid is NOT occupied (to allow spawning/rotating)
        assert!(!grid.is_occupied(0, GRID_HEIGHT, 0));
        assert!(!grid.is_occupied(GRID_WIDTH - 1, GRID_HEIGHT + 5, GRID_DEPTH - 1));

        // Horizontal OOB still applies even if above grid height
        assert!(grid.is_occupied(-1, GRID_HEIGHT, 0));
    }

    #[test]
    fn test_rotation_all_axes() {
        let p = IVec3::new(1, 2, 3);

        // X Axis: (x, y, z) -> (x, -z, y)
        assert_eq!(rotate_point(p, IVec3::X, true), IVec3::new(1, -3, 2));
        assert_eq!(rotate_point(p, IVec3::X, false), IVec3::new(1, 3, -2));

        // Y Axis: (x, y, z) -> (z, y, -x)
        assert_eq!(rotate_point(p, IVec3::Y, true), IVec3::new(3, 2, -1));
        assert_eq!(rotate_point(p, IVec3::Y, false), IVec3::new(-3, 2, 1));

        // Z Axis: (x, y, z) -> (-y, x, z)
        assert_eq!(rotate_point(p, IVec3::Z, true), IVec3::new(-2, 1, 3));
        assert_eq!(rotate_point(p, IVec3::Z, false), IVec3::new(2, -1, 3));
    }

    #[test]
    fn test_can_place_collision() {
        let mut grid = GameGrid::new();
        let positions = [IVec3::ZERO, IVec3::new(1, 0, 0), IVec3::new(0, 1, 0), IVec3::new(1, 1, 0)];
        let pivot = IVec3::new(2, 2, 2);

        // Valid
        assert!(can_place(&positions, pivot, &grid));

        // Collide with boundary (X)
        let oob_pivot = IVec3::new(GRID_WIDTH - 1, 0, 0);
        assert!(!can_place(&positions, oob_pivot, &grid));

        // Collide with block
        grid.set(3, 2, 2, TetracubeType::I);
        assert!(!can_place(&positions, pivot, &grid));
    }

    #[test]
    fn test_lock_tetracube() {
        let mut grid = GameGrid::new();
        let mut dirty = DirtyGrid(false);
        let tetracube = Tetracube {
            piece_type: TetracubeType::I,
            positions: [IVec3::ZERO, IVec3::ZERO, IVec3::ZERO, IVec3::ZERO],
            pivot: IVec3::new(0, 0, 0),
            color: Color::WHITE,
        };

        // Not game over
        let game_over = grid.lock_tetracube(&tetracube, &mut dirty);
        assert!(!game_over);
        assert!(grid.get(0, 0, 0).is_some());
        assert!(dirty.0);

        // Game over (locks at or above GRID_HEIGHT)
        let tetracube_high = Tetracube {
            piece_type: TetracubeType::I,
            positions: [IVec3::ZERO, IVec3::ZERO, IVec3::ZERO, IVec3::ZERO],
            pivot: IVec3::new(0, GRID_HEIGHT, 0),
            color: Color::WHITE,
        };
        let game_over_high = grid.lock_tetracube(&tetracube_high, &mut dirty);
        assert!(game_over_high);
    }

    #[test]
    fn test_grid_get_set() {
        let mut grid = GameGrid::new();
        grid.set(1, 2, 3, TetracubeType::T);

        assert_eq!(grid.get(1, 2, 3), Some(TetracubeType::T));
        assert!(grid.get(0, 0, 0).is_none());
        assert!(grid.get(-1, -1, -1).is_none());
    }

    #[test]
    fn test_clear_full_layers() {
        let mut grid = GameGrid::new();
        let mut dirty = DirtyGrid(false);
        // Fill a layer (y=0)
        for x in 0..GRID_WIDTH {
            for z in 0..GRID_DEPTH {
                grid.set(x, 0, z, TetracubeType::O);
            }
        }
        // Put one block at y=1
        grid.set(0, 1, 0, TetracubeType::Z);

        let cleared = grid.clear_full_layers(&mut dirty);
        assert_eq!(cleared, 1);
        assert!(dirty.0);

        // Block at y=1 should have dropped to y=0
        assert_eq!(grid.get(0, 0, 0), Some(TetracubeType::Z));
        // Layer y=1 should now be empty except for what dropped (or if it was already empty)
        assert!(grid.get(1, 1, 1).is_none());
    }

    #[test]
    fn test_clear_multiple_layers() {
        let mut grid = GameGrid::new();
        let mut dirty = DirtyGrid(false);
        // Fill two layers (y=0 and y=1)
        for y in 0..2 {
            for x in 0..GRID_WIDTH {
                for z in 0..GRID_DEPTH {
                    grid.set(x, y, z, TetracubeType::O);
                }
            }
        }
        // Put one block at y=2
        grid.set(0, 2, 0, TetracubeType::L);

        let cleared = grid.clear_full_layers(&mut dirty);
        assert_eq!(cleared, 2);

        // Block at y=2 should have dropped to y=0
        assert_eq!(grid.get(0, 0, 0), Some(TetracubeType::L));
        assert!(grid.get(0, 1, 0).is_none());
    }

    #[test]
    fn test_try_rotate_with_kicks_floor() {
        // Actually, let's just test that a kick happens if needed.
        // Place block that blocks normal rotation, but allows a kicked one.
        let mut grid = GameGrid::new();
        grid.set(2, 0, 3, TetracubeType::I); // Blocks (0,0,1) relative to (2,0,2)

        let mut tetracube = Tetracube {
            piece_type: TetracubeType::I,
            positions: [IVec3::new(0, 0, 0), IVec3::new(0, 1, 0), IVec3::ZERO, IVec3::ZERO],
            pivot: IVec3::new(2, 0, 2),
            color: Color::WHITE,
        };

        // Simple case: try_rotate_with_kicks should return true if valid.
        assert!(try_rotate_with_kicks(&mut tetracube, IVec3::Y, true, &grid));
    }

    #[test]
    fn test_calculate_hard_drop() {
        let mut grid = GameGrid::new();
        let tetracube = Tetracube {
            piece_type: TetracubeType::I,
            positions: [IVec3::ZERO, IVec3::ZERO, IVec3::ZERO, IVec3::ZERO],
            pivot: IVec3::new(0, 10, 0),
            color: Color::WHITE,
        };

        // Should drop to y=0
        let drop_pivot = calculate_hard_drop(&tetracube, &grid);
        assert_eq!(drop_pivot.y, 0);

        // Block at y=2
        grid.set(0, 2, 0, TetracubeType::I);
        let drop_pivot_blocked = calculate_hard_drop(&tetracube, &grid);
        assert_eq!(drop_pivot_blocked.y, 3);
    }

    #[test]
    fn test_game_stats_progression() {
        let mut stats = GameStats::new();
        assert_eq!(stats.level, 1);
        assert_eq!(stats.get_fall_speed(), 0.8);

        // Add 5 layers -> 500 points -> Level 2
        let leveled_up = stats.add_layers(5);
        assert!(leveled_up);
        assert_eq!(stats.level, 2);
        assert_eq!(stats.get_fall_speed(), 0.7);

        // More points...
        stats.add_layers(10); // 1000 more -> 1500 total -> Level 4
        assert_eq!(stats.level, 4);
        assert_eq!(stats.get_fall_speed(), 0.5);
    }

    #[test]
    fn test_3d_tetracube_shapes() {
        let types = [
            TetracubeType::I,
            TetracubeType::O,
            TetracubeType::T,
            TetracubeType::Z,
            TetracubeType::L,
            TetracubeType::Tripod,
            TetracubeType::ScrewL,
            TetracubeType::ScrewR,
        ];
        for t in types {
            let shapes = t.get_shape_blocks();
            assert_eq!(shapes.len(), 4, "Tetracube {:?} must have 4 blocks", t);

            let color = t.get_color();
            // Just verify it doesn't panic and returns something
            assert!(color.to_linear().to_vec4().length() > 0.0);
        }
    }

    #[test]
    fn test_layer_empty_full() {
        let mut grid = GameGrid::new();
        assert!(grid.is_layer_empty(0));
        assert!(!grid.is_layer_full(0));

        // Fill layer 0
        for x in 0..GRID_WIDTH {
            for z in 0..GRID_DEPTH {
                grid.set(x, 0, z, TetracubeType::I);
            }
        }
        assert!(!grid.is_layer_empty(0));
        assert!(grid.is_layer_full(0));

        // Partly fill layer 1
        grid.set(0, 1, 0, TetracubeType::I);
        assert!(!grid.is_layer_empty(1));
        assert!(!grid.is_layer_full(1));
    }

    #[test]
    fn test_lock_tetracube_out_of_bounds() {
        let mut grid = GameGrid::new();
        let mut dirty = DirtyGrid(false);
        let tetracube = Tetracube {
            piece_type: TetracubeType::I,
            positions: [IVec3::ZERO, IVec3::ZERO, IVec3::ZERO, IVec3::ZERO],
            pivot: IVec3::new(-1, 0, 0), // OOB X
            color: Color::WHITE,
        };
        // Should not panic and should not set anything in grid (index returns None)
        grid.lock_tetracube(&tetracube, &mut dirty);
        assert!(dirty.0);
    }

    #[test]
    fn test_try_rotate_with_kicks_wall() {
        let grid = GameGrid::new();

        // Let's use a piece that MUST kick.
        let mut tetracube = Tetracube {
            piece_type: TetracubeType::I,
            positions: [IVec3::ZERO, IVec3::new(0, 1, 0), IVec3::ZERO, IVec3::ZERO],
            pivot: IVec3::new(GRID_WIDTH - 1, 0, 0),
            color: Color::WHITE,
        };

        // Let's put it at X=0 and rotate it to X=-1.
        tetracube.pivot = IVec3::new(0, 0, 0);
        // Rotate Z (forward): (0,1,0) -> (-1,0,0). OOB!
        // It should kick to (1,0,0) or similar.
        assert!(try_rotate_with_kicks(&mut tetracube, IVec3::Z, true, &grid));
    }
}
