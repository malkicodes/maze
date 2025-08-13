pub mod generators;
pub mod solvers;

use std::time::Instant;
use sfml::{
    graphics::{Drawable, RectangleShape, Shape, Transformable},
    system::Vector2f,
};
use crate::Direction;

use crate::consts::{CELL_COLOR, CELL_SIZE, EMPTY_CELL_COLOR, WALL_COLOR, WALL_WIDTH};

#[derive(Debug)]
pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<u8>,
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![0; width * height];

        Self{
            width,
            height,
            cells,
        }
    }

    pub fn from_str(data: &str) -> Result<Self, String> {
        let segments: Vec<&str> = data.split(';').collect();
        
        if segments.len() != 3 {
            return Err(String::from("invalid segment count"));
        }

        let width = segments[0].parse::<usize>();
        if let Err(e) = width {
            return Err(format!("invalid width: {e}"));
        }

        let height = segments[1].parse::<usize>();
        if let Err(e) = height {
            return Err(format!("invalid height: {e}"));
        }

        let width = width.unwrap();
        let height = height.unwrap();

        let mut cells = vec![0; width * height];

        let cell_data = segments[2];

        if cell_data.len() != width * height {
            return Err(String::from("invalid cell data length"));
        }

        for (i, data) in cell_data.chars().enumerate() {
            cells[i] = data as u8 & 0b00001111;
        }
        
        Ok(Self{
            width,
            height,
            cells,
        })
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
            panic!("i {} larger than cells array length {}", i, self.cells.len())
        } else {
            self.cells[i]
        }
    }

    pub fn get_bounds(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn as_str(&self) -> String {
        let data = String::from_utf8(self.cells.iter().map(|a| 0b01000000 | a).collect()).unwrap();

        format!("{};{};{}", self.width, self.height, data)
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

    pub fn generate(&mut self, generator: &mut dyn MazeGenerator) {
        let mut step_count: usize = 0;
        
        let start = Instant::now();
        while !generator.step(self) {
            step_count += 1;
        }
        let duration = start.elapsed();

        println!("Generating maze took {} steps and {:?}", step_count, duration);
    }

    pub fn get_neighbors(&self, (x, y): (usize, usize)) -> Vec<(usize, usize, Direction)>  {
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

    pub fn get_travellable_neighbors(&self, (x, y): (usize, usize)) -> Vec<(usize, usize, Direction)> {
        let mut neighbors: Vec<(usize, usize, Direction)> = vec![];
        let value = self.get(x, y);

        if x > 0 && value & Direction::LEFT as u8 != 0 {
            neighbors.push((x - 1, y, Direction::LEFT));
        }

        if x + 1 < self.width && value & Direction::RIGHT as u8 != 0 {
            neighbors.push((x + 1, y, Direction::RIGHT));
        }

        if y > 0 && value & Direction::UP as u8 != 0 {
            neighbors.push((x, y - 1, Direction::UP));
        }

        if y + 1 < self.height && value & Direction::DOWN as u8 != 0  {
            neighbors.push((x, y + 1, Direction::DOWN));
        }

        neighbors
    }
}

impl Drawable for Maze {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.clear(WALL_COLOR);

        let mut empty_rect = RectangleShape::with_size(
            (
                (CELL_SIZE - WALL_WIDTH * 2) as f32,
                (CELL_SIZE - WALL_WIDTH * 2) as f32,
            )
                .into(),
        );
        empty_rect.set_origin((
            CELL_SIZE as f32 / 2. - WALL_WIDTH as f32,
            CELL_SIZE as f32 / 2. - WALL_WIDTH as f32,
        ));
        empty_rect.set_fill_color(EMPTY_CELL_COLOR);

        let mut up_rect = RectangleShape::with_size(
            (
                (CELL_SIZE - WALL_WIDTH * 2) as f32,
                (CELL_SIZE - WALL_WIDTH) as f32,
            )
                .into(),
        );
        up_rect.set_origin((
            CELL_SIZE as f32 / 2. - WALL_WIDTH as f32,
            CELL_SIZE as f32 / 2.,
        ));
        up_rect.set_fill_color(CELL_COLOR);

        let mut down_rect = up_rect.clone();
        down_rect.set_origin((
            CELL_SIZE as f32 / 2. - WALL_WIDTH as f32,
            CELL_SIZE as f32 / 2. - WALL_WIDTH as f32,
        ));
        down_rect.set_fill_color(CELL_COLOR);

        let mut left_rect = RectangleShape::with_size(
            up_rect.size().perpendicular()
        );
        left_rect.set_origin((
            CELL_SIZE as f32 / -2. + WALL_WIDTH as f32,
            CELL_SIZE as f32 / 2. - WALL_WIDTH as f32,
        ));
        left_rect.set_fill_color(CELL_COLOR);

        let mut right_rect = left_rect.clone();
        right_rect.set_origin((
            CELL_SIZE as f32 / -2.,
            CELL_SIZE as f32 / 2. - WALL_WIDTH as f32,
        ));
        right_rect.set_fill_color(CELL_COLOR);

        for y in 0..self.height {
            for x in 0..self.width {
                let position = Vector2f::new(
                    ((x * 2 + 1) * CELL_SIZE) as f32 / 2.,
                    ((y * 2 + 1) * CELL_SIZE) as f32 / 2.,
                );
                let cell = self.get(x, y);

                if cell == 0 {
                    empty_rect.set_position(position);
                    target.draw_rectangle_shape(&empty_rect, rs);
                    continue
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

pub trait MazeGenerator: Drawable {
    fn step(&mut self, maze: &mut Maze) -> bool;
}

pub trait MazeSolver: Drawable {
    fn new(bounds: (usize, usize)) -> Self;
    fn step(&mut self, maze: &Maze) -> Option<&Vec<(usize, usize)>>;
}
