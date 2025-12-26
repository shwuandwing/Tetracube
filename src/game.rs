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
    pub shape: TetrominoType,
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
        if let Some(idx) = Self::index(x, y, z) {
            self.grid[idx].is_some()
        } else {
            true // Out of bounds is considered "occupied" for collision purposes (except top)
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
        
        // Out of bounds is occupied
        assert!(grid.is_occupied(-1, 5, 2));
    }
}
