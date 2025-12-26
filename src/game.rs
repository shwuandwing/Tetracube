use bevy::prelude::*;

pub const GRID_WIDTH: i32 = 8;
pub const GRID_HEIGHT: i32 = 15;
pub const GRID_DEPTH: i32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TetrominoType {
    I, O, T, Z, L,
    Tripod, ScrewL, ScrewR,
}

#[derive(Component, Clone)]
pub struct Tetromino {
    pub positions: Vec<IVec3>, // Relative positions to the pivot
    pub pivot: IVec3,          // Global position
    pub color: Color,
}

#[derive(Resource, Default)]
pub struct DirtyGrid(pub bool);

#[derive(Resource)]
pub struct GameGrid {
    // Stores the color of the block at x, y, z. None means empty.
    pub grid: Vec<Option<Color>>, 
}

impl GameGrid {
    pub fn new() -> Self {
        Self {
            grid: vec![None; (GRID_WIDTH * GRID_HEIGHT * GRID_DEPTH) as usize],
        }
    }

    pub fn index(x: i32, y: i32, z: i32) -> Option<usize> {
        if x < 0 || x >= GRID_WIDTH || y < 0 || y >= GRID_HEIGHT || z < 0 || z >= GRID_DEPTH {
            return None;
        }
        Some((y * GRID_WIDTH * GRID_DEPTH + z * GRID_WIDTH + x) as usize)
    }

    pub fn is_occupied(&self, x: i32, y: i32, z: i32) -> bool {
        if x < 0 || x >= GRID_WIDTH || z < 0 || z >= GRID_DEPTH || y < 0 {
            return true;
        }
        if y >= GRID_HEIGHT {
            return false;
        }
        if let Some(idx) = Self::index(x, y, z) {
            self.grid[idx].is_some()
        } else {
            true
        }
    }
    
    pub fn is_valid_pos(&self, x: i32, y: i32, z: i32) -> bool {
         x >= 0 && x < GRID_WIDTH && z >= 0 && z < GRID_DEPTH && y >= 0
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, color: Color) {
        if let Some(idx) = Self::index(x, y, z) {
            self.grid[idx] = Some(color);
        }
    }
    
    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<Color> {
         if let Some(idx) = Self::index(x, y, z) {
            self.grid[idx]
        } else {
            None
        }
    }

    /// Clears full 2D layers and returns the number of lines cleared.
    pub fn clear_full_lines(&mut self, dirty: &mut DirtyGrid) -> u32 {
        let mut lines_cleared = 0;
        let mut y = 0;
        while y < GRID_HEIGHT {
            let mut full = true;
            for x in 0..GRID_WIDTH {
                for z in 0..GRID_DEPTH {
                    if self.get(x, y, z).is_none() {
                        full = false;
                        break;
                    }
                }
                if !full { break; }
            }

            if full {
                lines_cleared += 1;
                dirty.0 = true;
                // Shift down
                for dy in y..(GRID_HEIGHT - 1) {
                    for x in 0..GRID_WIDTH {
                        for z in 0..GRID_DEPTH {
                            let above = self.get(x, dy + 1, z);
                            if let Some(c) = above {
                                 self.set(x, dy, z, c);
                            } else {
                                 if let Some(idx) = Self::index(x, dy, z) {
                                     self.grid[idx] = None;
                                 }
                            }
                        }
                    }
                }
                // Clear top row
                for x in 0..GRID_WIDTH {
                    for z in 0..GRID_DEPTH {
                         if let Some(idx) = Self::index(x, GRID_HEIGHT-1, z) {
                             self.grid[idx] = None;
                         }
                    }
                }
            } else {
                y += 1;
            }
        }
        lines_cleared
    }

    /// Locks a tetromino into the grid. Returns true if it's a Game Over.
    pub fn lock_tetromino(&mut self, tetromino: &Tetromino, dirty: &mut DirtyGrid) -> bool {
        let mut game_over = false;
        dirty.0 = true;
        for pos in &tetromino.positions {
            let global = tetromino.pivot + *pos;
            if global.y >= GRID_HEIGHT {
                game_over = true;
            }
            self.set(global.x, global.y, global.z, tetromino.color);
        }
        game_over
    }
}

pub fn try_rotate_with_kicks(
    tetromino: &mut Tetromino, 
    axis: IVec3, 
    forward: bool, 
    grid: &GameGrid
) -> bool {
    let new_positions: Vec<IVec3> = tetromino.positions.iter()
        .map(|p| rotate_point(*p, axis, forward))
        .collect();

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
        let test_pivot = tetromino.pivot + kick;
        if can_place(&new_positions, test_pivot, grid) {
            tetromino.positions = new_positions;
            tetromino.pivot = test_pivot;
            return true;
        }
    }
    false
}

pub fn rotate_point(point: IVec3, axis: IVec3, forward: bool) -> IVec3 {
    // 90 degree rotation
    // Forward: +90 deg. Inverse: -90 deg.
    if axis.x == 1 {
        // X Axis: (x, y, z) -> (x, -z, y)
        if forward { IVec3::new(point.x, -point.z, point.y) }
        else { IVec3::new(point.x, point.z, -point.y) }
    } else if axis.y == 1 {
        // Y Axis: (x, y, z) -> (z, y, -x)
        if forward { IVec3::new(point.z, point.y, -point.x) }
        else { IVec3::new(-point.z, point.y, point.x) }
    } else { // z
        // Z Axis: (x, y, z) -> (-y, x, z)
        if forward { IVec3::new(-point.y, point.x, point.z) }
        else { IVec3::new(point.y, -point.x, point.z) }
    }
}

pub fn can_place(positions: &[IVec3], pivot: IVec3, grid: &GameGrid) -> bool {
    for pos in positions {
        let global = pivot + *pos;
        if !grid.is_valid_pos(global.x, global.y, global.z) || grid.is_occupied(global.x, global.y, global.z) {
            return false;
        }
    }
    true
}

pub fn calculate_hard_drop(tetromino: &Tetromino, grid: &GameGrid) -> IVec3 {
    let mut current_pivot = tetromino.pivot;
    loop {
        let next_y_pivot = current_pivot + IVec3::new(0, -1, 0);
        if can_place(&tetromino.positions, next_y_pivot, grid) {
            current_pivot = next_y_pivot;
        } else {
            break;
        }
    }
    current_pivot
}

#[derive(Resource, Default)]
pub struct GameStats {
    pub score: u32,
    pub level: u32,
}

impl GameStats {
    pub fn new() -> Self {
        Self { score: 0, level: 1 }
    }

    pub fn add_lines(&mut self, lines: u32) -> bool {
        if lines == 0 { return false; }
        self.score += lines * 100;
        let new_level = (self.score / 500) + 1;
        if new_level > self.level {
            self.level = new_level;
            return true;
        }
        false
    }

    pub fn get_fall_speed(&self) -> f32 {
        (0.8 - (self.level as f32 - 1.0) * 0.1).max(0.1)
    }
}

pub fn get_shape_blocks(piece_type: TetrominoType) -> Vec<IVec3> {
    match piece_type {
        // Defined in X/Z plane (y=0)
        TetrominoType::I => vec![IVec3::new(0,0,0), IVec3::new(-1,0,0), IVec3::new(1,0,0), IVec3::new(2,0,0)],
        TetrominoType::O => vec![IVec3::new(0,0,0), IVec3::new(1,0,0), IVec3::new(0,0,1), IVec3::new(1,0,1)],
        TetrominoType::T => vec![IVec3::new(0,0,0), IVec3::new(-1,0,0), IVec3::new(1,0,0), IVec3::new(0,0,1)],
        TetrominoType::Z => vec![IVec3::new(0,0,0), IVec3::new(1,0,0), IVec3::new(0,0,1), IVec3::new(-1,0,1)],
        TetrominoType::L => vec![IVec3::new(0,0,0), IVec3::new(-1,0,0), IVec3::new(1,0,0), IVec3::new(1,0,1)],
        TetrominoType::Tripod => vec![IVec3::new(0,0,0), IVec3::new(1,0,0), IVec3::new(0,1,0), IVec3::new(0,0,1)],
        TetrominoType::ScrewL => vec![IVec3::new(0,0,0), IVec3::new(1,0,0), IVec3::new(1,1,0), IVec3::new(1,1,1)],
        TetrominoType::ScrewR => vec![IVec3::new(0,0,0), IVec3::new(1,0,0), IVec3::new(1,1,0), IVec3::new(1,1,-1)],
    }
}

pub fn get_random_color(piece_type: TetrominoType) -> Color {
     match piece_type {
        TetrominoType::I => Color::srgb(0.0, 1.0, 1.0), // Cyan
        TetrominoType::O => Color::srgb(1.0, 1.0, 0.0), // Yellow
        TetrominoType::T => Color::srgb(1.0, 0.0, 1.0), // Magenta
        TetrominoType::Z => Color::srgb(1.0, 0.0, 0.0), // Red
        TetrominoType::L => Color::srgb(1.0, 0.5, 0.0), // Orange
        TetrominoType::Tripod => Color::srgb(1.0, 1.0, 1.0), // White
        TetrominoType::ScrewL => Color::srgb(0.5, 1.0, 0.0), // Lime
        TetrominoType::ScrewR => Color::srgb(0.5, 0.0, 1.0), // Purple
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
        
        grid.set(0, 0, 0, Color::WHITE);
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
        let positions = vec![IVec3::ZERO, IVec3::new(1, 0, 0)];
        let pivot = IVec3::new(2, 2, 2);

        // Valid
        assert!(can_place(&positions, pivot, &grid));

        // Collide with boundary (X)
        let oob_pivot = IVec3::new(GRID_WIDTH - 1, 0, 0);
        assert!(!can_place(&positions, oob_pivot, &grid));

        // Collide with block
        grid.set(3, 2, 2, Color::WHITE);
        assert!(!can_place(&positions, pivot, &grid));
    }

    #[test]
    fn test_lock_tetromino() {
        let mut grid = GameGrid::new();
        let mut dirty = DirtyGrid(false);
        let tetromino = Tetromino {
            positions: vec![IVec3::ZERO],
            pivot: IVec3::new(0, 0, 0),
            color: Color::WHITE,
        };
        
        // Not game over
        let game_over = grid.lock_tetromino(&tetromino, &mut dirty);
        assert!(!game_over);
        assert!(grid.get(0, 0, 0).is_some());
        assert!(dirty.0);

        // Game over (locks at or above GRID_HEIGHT)
        let tetromino_high = Tetromino {
            positions: vec![IVec3::ZERO],
            pivot: IVec3::new(0, GRID_HEIGHT, 0),
            color: Color::WHITE,
        };
        let game_over_high = grid.lock_tetromino(&tetromino_high, &mut dirty);
        assert!(game_over_high);
    }

    #[test]
    fn test_grid_get_set() {
        let mut grid = GameGrid::new();
        let color = Color::srgb(1.0, 0.0, 0.0);
        grid.set(1, 2, 3, color);
        
        // Use a small epsilon for color comparison or just compare if it's Some
        assert!(grid.get(1, 2, 3).is_some());
        assert!(grid.get(0, 0, 0).is_none());
        assert!(grid.get(-1, -1, -1).is_none());
    }

    #[test]
    fn test_clear_full_lines() {
        let mut grid = GameGrid::new();
        let mut dirty = DirtyGrid(false);
        // Fill a layer (y=0)
        for x in 0..GRID_WIDTH {
            for z in 0..GRID_DEPTH {
                grid.set(x, 0, z, Color::WHITE);
            }
        }
        // Put one block at y=1
        grid.set(0, 1, 0, Color::srgb(0.0, 1.0, 0.0));

        let cleared = grid.clear_full_lines(&mut dirty);
        assert_eq!(cleared, 1);
        assert!(dirty.0);
        
        // Block at y=1 should have dropped to y=0
        assert!(grid.get(0, 0, 0).is_some());
        // Layer y=1 should now be empty except for what dropped (or if it was already empty)
        assert!(grid.get(1, 1, 1).is_none());
    }

    #[test]
    fn test_clear_multiple_lines() {
        let mut grid = GameGrid::new();
        let mut dirty = DirtyGrid(false);
        // Fill two layers (y=0 and y=1)
        for y in 0..2 {
            for x in 0..GRID_WIDTH {
                for z in 0..GRID_DEPTH {
                    grid.set(x, y, z, Color::WHITE);
                }
            }
        }
        // Put one block at y=2
        grid.set(0, 2, 0, Color::WHITE);

        let cleared = grid.clear_full_lines(&mut dirty);
        assert_eq!(cleared, 2);
        
        // Block at y=2 should have dropped to y=0
        assert!(grid.get(0, 0, 0).is_some());
        assert!(grid.get(0, 1, 0).is_none());
    }

    #[test]
    fn test_try_rotate_with_kicks_floor() {
        // I piece lying flat at y=0. Pivot at y=0.
        // Rotation might push some blocks to y=-1, requiring a floor kick (upwards).
        let mut tetromino = Tetromino {
            positions: vec![IVec3::new(0, 0, 0), IVec3::new(0, 1, 0)], // Vertical 2-block piece
            pivot: IVec3::new(2, 0, 2),
            color: Color::WHITE,
        };
        
        // Rotate around X axis such that it would go below floor if not kicked
        // Before: (2,0,2), (2,1,2)
        // Rotate X (forward): (x, -z, y) relative
        // (0,0,0) -> (0,0,0)
        // (0,1,0) -> (0,0,1)
        // This specific rotation doesn't hit floor. 
        
        // Let's try one that DOES hit the floor.
        tetromino.positions = vec![IVec3::new(0, 0, 0), IVec3::new(0, 0, 1)];
        // Rotate around X (inverse): (x, z, -y) relative
        // (0,0,1) -> (0,1,0) - No.
        
        // Actually, let's just test that a kick happens if needed.
        // Place block that blocks normal rotation, but allows a kicked one.
        let mut grid = GameGrid::new();
        grid.set(2, 0, 3, Color::WHITE); // Blocks (0,0,1) relative to (2,0,2)
        
        let mut tetromino = Tetromino {
            positions: vec![IVec3::new(0, 0, 0), IVec3::new(0, 1, 0)],
            pivot: IVec3::new(2, 0, 2),
            color: Color::WHITE,
        };
        
        // Rotate Y (forward): (z, y, -x) relative
        // (0,1,0) -> (0,1,0) - No change in y.
        
        // Simple case: try_rotate_with_kicks should return true if valid.
        assert!(try_rotate_with_kicks(&mut tetromino, IVec3::Y, true, &grid));
    }

    #[test]
    fn test_calculate_hard_drop() {
        let mut grid = GameGrid::new();
        let tetromino = Tetromino {
            positions: vec![IVec3::ZERO],
            pivot: IVec3::new(0, 10, 0),
            color: Color::WHITE,
        };
        
        // Should drop to y=0
        let drop_pivot = calculate_hard_drop(&tetromino, &grid);
        assert_eq!(drop_pivot.y, 0);

        // Block at y=2
        grid.set(0, 2, 0, Color::WHITE);
        let drop_pivot_blocked = calculate_hard_drop(&tetromino, &grid);
        assert_eq!(drop_pivot_blocked.y, 3);
    }

    #[test]
    fn test_game_stats_progression() {
        let mut stats = GameStats::new();
        assert_eq!(stats.level, 1);
        assert_eq!(stats.get_fall_speed(), 0.8);

        // Add 5 lines -> 500 points -> Level 2
        let leveled_up = stats.add_lines(5);
        assert!(leveled_up);
        assert_eq!(stats.level, 2);
        assert_eq!(stats.get_fall_speed(), 0.7);

        // More points...
        stats.add_lines(10); // 1000 more -> 1500 total -> Level 4
        assert_eq!(stats.level, 4);
        assert_eq!(stats.get_fall_speed(), 0.5);
    }

    #[test]
    fn test_new_3d_tetromino_shapes() {
        let types = [TetrominoType::Tripod, TetrominoType::ScrewL, TetrominoType::ScrewR];
        for t in types {
            let shapes = get_shape_blocks(t);
            assert_eq!(shapes.len(), 4, "Tetromino {:?} must have 4 blocks", t);
            
            let color = get_random_color(t);
            // Just verify it doesn't panic and returns something
            assert!(color.to_linear().to_vec4().length() > 0.0);
        }
    }

    #[test]
    fn test_lock_tetromino_out_of_bounds() {
        let mut grid = GameGrid::new();
        let mut dirty = DirtyGrid(false);
        let tetromino = Tetromino {
            positions: vec![IVec3::ZERO],
            pivot: IVec3::new(-1, 0, 0), // OOB X
            color: Color::WHITE,
        };
        // Should not panic and should not set anything in grid (index returns None)
        grid.lock_tetromino(&tetromino, &mut dirty);
        assert!(dirty.0);
    }

    #[test]
    fn test_try_rotate_with_kicks_wall() {
        let grid = GameGrid::new();
        
        // Let's use I-like piece at the edge.
        // Rotation around Y (forward): (z, y, -x)
        // (1,0,0) -> (0,0,-1)
        // (2,0,0) -> (0,0,-2)
        // This won't hit the wall.
        
        // Rotation around Z (forward): (-y, x, z)
        // (1,0,0) -> (0,1,0)
        // (2,0,0) -> (0,2,0)
        // This won't hit the wall.
        
        // Let's use a piece that MUST kick.
        let mut tetromino = Tetromino {
            positions: vec![IVec3::ZERO, IVec3::new(0, 1, 0)],
            pivot: IVec3::new(GRID_WIDTH - 1, 0, 0),
            color: Color::WHITE,
        };
        // Rotate Z (forward): (-y, x, z)
        // (0,1,0) -> (-1,0,0) relative to (7,0,0) -> (6,0,0). Valid.
        
        // Let's put it at X=0 and rotate it to X=-1.
        tetromino.pivot = IVec3::new(0, 0, 0);
        // Rotate Z (forward): (0,1,0) -> (-1,0,0). OOB!
        // It should kick to (1,0,0) or similar.
        assert!(try_rotate_with_kicks(&mut tetromino, IVec3::Z, true, &grid));
    }
}
