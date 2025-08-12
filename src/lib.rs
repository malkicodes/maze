pub mod maze;

pub mod consts {
    use sfml::graphics::Color;

    pub const MAZE_WIDTH: usize = 24;
    pub const MAZE_HEIGHT: usize = 24;
    pub const CELL_SIZE: usize = 24;
    pub const WALL_WIDTH: usize = 1;

    pub const WALL_COLOR: Color = Color::rgb(0, 0, 0);
    pub const CELL_COLOR: Color = Color::rgb(255, 255, 255);
    pub const EMPTY_CELL_COLOR: Color = Color::rgb(64, 64, 64);
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
