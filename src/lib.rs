pub mod maze;

pub mod consts {
    use std::sync::{LazyLock, RwLock};

    use sfml::graphics::Color;

    pub const DEFAULT_MAZE_WIDTH: u16 = 32;
    pub const DEFAULT_MAZE_HEIGHT: u16 = 32;
    pub const PREFERRED_SCREEN_SIZE: usize = 512;
    pub const WALL_WIDTH: usize = 1;
    
    pub const WALL_COLOR: Color = Color::rgb(0, 0, 0);
    pub const CELL_COLOR: Color = Color::rgb(255, 255, 255);
    pub const EMPTY_CELL_COLOR: Color = Color::rgb(64, 64, 64);
    
    pub static CELL_SIZE: LazyLock<RwLock<usize>> = LazyLock::new(|| RwLock::new(16));

    pub fn get_cell_size() -> usize {
        *CELL_SIZE.read().unwrap()
    }

    pub fn update_cell_size(bounds: &(usize, usize)) {
        let average_size = (bounds.0 + bounds.1) / 2;

        let mut w = CELL_SIZE.write().unwrap();
        *w = (PREFERRED_SCREEN_SIZE / average_size).max(5);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    UP = 0b0001,
    RIGHT = 0b0010,
    DOWN = 0b0100,
    LEFT = 0b1000,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::UP => Direction::DOWN,
            Direction::DOWN => Direction::UP,
            Direction::LEFT => Direction::RIGHT,
            Direction::RIGHT => Direction::LEFT,
        }
    }

    pub fn travel(&self, x: usize, y: usize) -> (usize, usize) {
        match self {
            Direction::UP => (x, y - 1),
            Direction::DOWN => (x, y + 1),
            Direction::LEFT => (x - 1, y),
            Direction::RIGHT => (x + 1, y),
        }
    }
}
