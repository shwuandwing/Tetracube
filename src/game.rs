use bevy::prelude::*;

pub const GRID_WIDTH: i32 = 8;
pub const GRID_HEIGHT: i32 = 15;
pub const GRID_DEPTH: i32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TetrominoType {
    I, O, T, S, Z, J, L
}

#[derive(Component, Clone)]
pub struct Tetromino {
    pub positions: Vec<IVec3>, // Relative positions to the pivot
    pub pivot: IVec3,          // Global position
    pub color: Color,
}

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

pub fn is_valid_rotation(positions: &Vec<IVec3>, pivot: IVec3, grid: &GameGrid) -> bool {
    for pos in positions {
        let global = pivot + *pos;
        if !grid.is_valid_pos(global.x, global.y, global.z) || grid.is_occupied(global.x, global.y, global.z) {
            return false;
        }
    }
    true
}

pub fn get_shape_blocks(piece_type: TetrominoType) -> Vec<IVec3> {
    match piece_type {
        // Defined in X/Z plane (y=0)
        TetrominoType::I => vec![IVec3::new(0,0,0), IVec3::new(-1,0,0), IVec3::new(1,0,0), IVec3::new(2,0,0)],
        TetrominoType::O => vec![IVec3::new(0,0,0), IVec3::new(1,0,0), IVec3::new(0,0,1), IVec3::new(1,0,1)],
        TetrominoType::T => vec![IVec3::new(0,0,0), IVec3::new(-1,0,0), IVec3::new(1,0,0), IVec3::new(0,0,1)],
        TetrominoType::S => vec![IVec3::new(0,0,0), IVec3::new(-1,0,0), IVec3::new(0,0,1), IVec3::new(1,0,1)],
        TetrominoType::Z => vec![IVec3::new(0,0,0), IVec3::new(1,0,0), IVec3::new(0,0,1), IVec3::new(-1,0,1)],
        TetrominoType::J => vec![IVec3::new(0,0,0), IVec3::new(-1,0,0), IVec3::new(1,0,0), IVec3::new(-1,0,1)],
        TetrominoType::L => vec![IVec3::new(0,0,0), IVec3::new(-1,0,0), IVec3::new(1,0,0), IVec3::new(1,0,1)],
    }
}

pub fn get_random_color(piece_type: TetrominoType) -> Color {
     match piece_type {
        TetrominoType::I => Color::srgb(0.0, 1.0, 1.0), // Cyan
        TetrominoType::O => Color::srgb(1.0, 1.0, 0.0), // Yellow
        TetrominoType::T => Color::srgb(1.0, 0.0, 1.0), // Magenta
        TetrominoType::S => Color::srgb(0.0, 1.0, 0.0), // Green
        TetrominoType::Z => Color::srgb(1.0, 0.0, 0.0), // Red
        TetrominoType::J => Color::srgb(0.0, 0.0, 1.0), // Blue
        TetrominoType::L => Color::srgb(1.0, 0.5, 0.0), // Orange
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
    fn test_is_valid_rotation_collision() {
        let mut grid = GameGrid::new();
        let positions = vec![IVec3::ZERO, IVec3::new(1, 0, 0)];
        let pivot = IVec3::new(2, 2, 2);

        // Valid
        assert!(is_valid_rotation(&positions, pivot, &grid));

        // Collide with boundary (X)
        let oob_pivot = IVec3::new(GRID_WIDTH - 1, 0, 0);
        assert!(!is_valid_rotation(&positions, oob_pivot, &grid));

        // Collide with block
        grid.set(3, 2, 2, Color::WHITE);
        assert!(!is_valid_rotation(&positions, pivot, &grid));
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
    fn test_get_shape_blocks_uniqueness() {
        use std::collections::HashSet;
        let shapes = [
            TetrominoType::I, TetrominoType::O, TetrominoType::T, 
            TetrominoType::S, TetrominoType::Z, TetrominoType::J, TetrominoType::L
        ];
        for s in shapes {
            let blocks = get_shape_blocks(s);
            let mut set = HashSet::new();
            for b in blocks {
                assert!(set.insert(b), "Duplicate block position in shape {:?}", s);
            }
        }
    }
}
