pub mod generators;
pub mod solvers;

use crate::Direction;
use sfml::{
    graphics::{Drawable, RectangleShape, Shape, Transformable},
    system::Vector2f,
};

use crate::consts::{get_cell_size, CELL_COLOR, EMPTY_CELL_COLOR, WALL_COLOR, WALL_WIDTH};

#[derive(Debug)]
pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<u8>,
}

impl Maze {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![0; width as usize * height as usize];

        Self {
            width: width as usize,
            height: height as usize,
            cells,
        }
    }

    pub fn from_data(data: &[u8]) -> Result<Self, String> {
        decode_maze(data)
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        if x >= self.width {
            panic!("x {} larger than width {}", x, self.width)
        } else if y >= self.height {
            panic!("y {} larger than width {}", y, self.height)
        } else {
            self.cells[y * self.width + x]
        }
    }

    pub fn geti(&self, i: usize) -> u8 {
        if i >= self.cells.len() {
            panic!(
                "i {} larger than cells array length {}",
                i,
                self.cells.len()
            )
        } else {
            self.cells[i]
        }
    }

    pub fn get_bounds(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn as_str(&self) -> Result<String, String> {
        encode_maze(self)
    }

    pub fn i_to_xy(&self, i: usize) -> (usize, usize) {
        self.geti(i);
        (i % self.width, i / self.width)
    }

    pub fn xy_to_i(&self, x: usize, y: usize) -> usize {
        self.get(x, y);
        y * self.width + x
    }

    pub fn open(&mut self, x: usize, y: usize, direction: Direction) {
        let cell = self.get(x, y);

        self.cells[y * self.width + x] = cell
            | match direction {
                Direction::UP => 0b0001,
                Direction::RIGHT => 0b0010,
                Direction::DOWN => 0b0100,
                Direction::LEFT => 0b1000,
            }
    }

    pub fn close(&mut self, x: usize, y: usize, direction: Direction) {
        let cell = self.get(x, y);

        self.cells[y * self.width + x] = cell
            & match direction {
                Direction::UP => 0b1110,
                Direction::RIGHT => 0b1101,
                Direction::DOWN => 0b1011,
                Direction::LEFT => 0b0111,
            }
    }

    pub fn carve(&mut self, x: usize, y: usize, direction: Direction) {
        self.open(x, y, direction);

        let (x, y) = direction.travel(x, y);
        self.open(x, y, direction.opposite());
    }

    pub fn delete(&mut self, x: usize, y: usize) {
        self.get(x, y);

        self.cells[y * self.width + x] = 0
    }

    pub fn get_neighbors(&self, (x, y): (usize, usize)) -> Vec<(usize, usize, Direction)> {
        let mut neighbors: Vec<(usize, usize, Direction)> = vec![];

        if x > 0 {
            neighbors.push((x - 1, y, Direction::LEFT));
        }

        if x + 1 < self.width {
            neighbors.push((x + 1, y, Direction::RIGHT));
        }

        if y > 0 {
            neighbors.push((x, y - 1, Direction::UP));
        }

        if y + 1 < self.height {
            neighbors.push((x, y + 1, Direction::DOWN));
        }

        neighbors
    }

    pub fn get_travellable_neighbors(
        &self,
        (x, y): (usize, usize),
    ) -> ([(usize, usize); 4], usize) {
        let value = self.get(x, y);

        let mut neighbors: [_; 4] = [(0, 0); 4];
        let mut neighbor_count = 0;

        if x > 0 && value & Direction::LEFT as u8 != 0 {
            neighbors[neighbor_count] = (x - 1, y);
            neighbor_count += 1;
        }

        if x + 1 < self.width && value & Direction::RIGHT as u8 != 0 {
            neighbors[neighbor_count] = (x + 1, y);
            neighbor_count += 1;
        }

        if y > 0 && value & Direction::UP as u8 != 0 {
            neighbors[neighbor_count] = (x, y - 1);
            neighbor_count += 1;
        }

        if y + 1 < self.height && value & Direction::DOWN as u8 != 0 {
            neighbors[neighbor_count] = (x, y + 1);
            neighbor_count += 1;
        }

        (neighbors, neighbor_count)
    }
}

impl Drawable for Maze {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.clear(WALL_COLOR);
        let cell_size = get_cell_size();

        let mut empty_rect = RectangleShape::with_size(
            (
                (cell_size - WALL_WIDTH * 2) as f32,
                (cell_size - WALL_WIDTH * 2) as f32,
            )
                .into(),
        );
        empty_rect.set_origin((
            cell_size as f32 / 2. - WALL_WIDTH as f32,
            cell_size as f32 / 2. - WALL_WIDTH as f32,
        ));
        empty_rect.set_fill_color(EMPTY_CELL_COLOR);

        let mut up_rect = RectangleShape::with_size(
            (
                (cell_size - WALL_WIDTH * 2) as f32,
                (cell_size - WALL_WIDTH) as f32,
            )
                .into(),
        );
        up_rect.set_origin((
            cell_size as f32 / 2. - WALL_WIDTH as f32,
            cell_size as f32 / 2.,
        ));
        up_rect.set_fill_color(CELL_COLOR);

        let mut down_rect = up_rect.clone();
        down_rect.set_origin((
            cell_size as f32 / 2. - WALL_WIDTH as f32,
            cell_size as f32 / 2. - WALL_WIDTH as f32,
        ));
        down_rect.set_fill_color(CELL_COLOR);

        let mut left_rect = RectangleShape::with_size(up_rect.size().perpendicular());
        left_rect.set_origin((
            cell_size as f32 / -2. + WALL_WIDTH as f32,
            cell_size as f32 / 2. - WALL_WIDTH as f32,
        ));
        left_rect.set_fill_color(CELL_COLOR);

        let mut right_rect = left_rect.clone();
        right_rect.set_origin((
            cell_size as f32 / -2.,
            cell_size as f32 / 2. - WALL_WIDTH as f32,
        ));
        right_rect.set_fill_color(CELL_COLOR);

        for y in 0..self.height {
            for x in 0..self.width {
                let position = Vector2f::new(
                    ((x * 2 + 1) * cell_size) as f32 / 2.,
                    ((y * 2 + 1) * cell_size) as f32 / 2.,
                );

                let cell = self.get(x, y);

                if cell == 0 {
                    empty_rect.set_position(position);
                    target.draw_rectangle_shape(&empty_rect, rs);
                    continue;
                }

                if (cell & Direction::UP as u8) != 0 {
                    up_rect.set_position(position);
                    target.draw_rectangle_shape(&up_rect, rs);
                }

                if (cell & Direction::DOWN as u8) != 0 {
                    down_rect.set_position(position);
                    target.draw_rectangle_shape(&down_rect, rs);
                }

                if (cell & Direction::LEFT as u8) != 0 {
                    left_rect.set_position(position);
                    target.draw_rectangle_shape(&left_rect, rs);
                }

                if (cell & Direction::RIGHT as u8) != 0 {
                    right_rect.set_position(position);
                    target.draw_rectangle_shape(&right_rect, rs);
                }
            }
        }
    }
}

fn encode_maze(maze: &Maze) -> Result<String, String> {
    let mut data = vec![];

    if maze.width > u16::MAX as usize {
        return Err(String::from("maze width too large"));
    }

    let width: u16 = maze.width.try_into().unwrap();
    data.extend(width.to_be_bytes());

    let default = 0;

    for i in 0..maze.cells.len().div_ceil(2) {
        let cell1 = *maze.cells.get(i * 2).unwrap_or(&default);
        let cell2 = *maze.cells.get(i * 2 + 1).unwrap_or(&default);

        data.push(cell1 << 4 | cell2);
    }

    Ok(unsafe { String::from_utf8_unchecked(data) })
}

fn decode_maze(data: &[u8]) -> Result<Maze, String> {
    let cell_data = &data[2..];

    let width = (((data[0] as u16) << 8) + data[1] as u16) as usize;
    let cell_count = if cell_data.last().unwrap() & 0x0f == 0 {
        cell_data.len() * 2 - 1
    } else {
        cell_data.len() * 2
    };
    let height = cell_count / width;

    let mut cells: Vec<u8> = vec![];

    for (i, cell_pair) in cell_data.iter().enumerate() {
        cells.push((*cell_pair) >> 4);
        if i * 2 + 1 < cell_count {
            cells.push((*cell_pair) & 0x0f);
        }
    }

    Ok(Maze {
        width,
        height,
        cells,
    })
}

pub trait MazeGenerator: Drawable {
    fn step(&mut self, maze: &mut Maze) -> bool;
}

pub trait MazeSolver: Drawable {
    fn new(bounds: (usize, usize)) -> Self;
    fn step(&mut self, maze: &Maze) -> Option<&Vec<(usize, usize)>>;
}
