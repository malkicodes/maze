use std::vec;

use rand::{rng, Rng};
use sfml::graphics::{CircleShape, Color, Drawable, PrimitiveType, Shape, Transformable, Vertex, VertexBuffer, VertexBufferUsage};

use crate::consts::*;
use crate::maze::{Maze, MazeGenerator};
use crate::Direction;

pub struct RandomDFS {
    stack: Vec<(usize, usize)>,
}

impl RandomDFS {
    pub fn new(bounds: (usize, usize)) -> Self {
        Self{
            stack: vec![(
                rng().random_range(0..bounds.0),
                rng().random_range(0..bounds.1),
            )]
        }
    }
}

impl Drawable for RandomDFS {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let mut polyline = VertexBuffer::new(PrimitiveType::LINE_STRIP, self.stack.len(), VertexBufferUsage::DYNAMIC).unwrap();

        let points: Vec<Vertex> = self.stack.iter().map(
            |(x, y)| Vertex::with_pos_color((((*x * 2 + 1) * CELL_SIZE / 2) as f32, ((*y * 2 + 1) * CELL_SIZE / 2) as f32).into(), Color::RED)
        ).collect();

        polyline.update(&points, 0).unwrap();

        target.draw_vertex_buffer(&polyline, rs);
    }
}

impl MazeGenerator for RandomDFS {
    fn step(&mut self, maze: &mut super::Maze) -> bool {
        let last_pos = self.stack.last();

        match last_pos {
            None => return true,
            Some(_) => {}
        }

        let pos = *last_pos.unwrap();

        let neighbors = maze.get_neighbors(pos);
        let possible_next: Vec<_> = neighbors.iter().filter(
            |(x, y, _)| maze.get(*x, *y) == 0
        ).collect();

        if possible_next.len() == 0 {
            self.stack.pop();

            self.stack.len() == 0
        } else {
            let next = possible_next[rng().random_range(..possible_next.len())];

            self.stack.push((next.0, next.1));
            maze.carve(pos.0, pos.1, next.2);

            false
        }
    }
}

pub struct Wilson {
    walk: Vec<(usize, usize)>,
    first_walk_target: Option<(usize, usize)>,
    opposite_of_last_direction: Option<Direction>,

    current_walk_steps: usize,
}

impl Wilson {
    pub fn new(bounds: (usize, usize)) -> Self {
        let start = (rng().random_range(..bounds.0), rng().random_range(..bounds.1));

        let end = loop{
            let next = (rng().random_range(..bounds.0), rng().random_range(..bounds.1));

            if next.0 == start.0 && next.1 == start.1 {
                continue;
            }

            break next
        };

        Self{
            walk: vec![start],
            first_walk_target: Some(end),
            opposite_of_last_direction: None,
            current_walk_steps: 0,
        }
    }

    fn pos_in_stack(&self, pos: (usize, usize)) -> Option<usize> {
        self.walk.iter().position(|v| pos.0 == v.0 && pos.1 == v.1)
    }

    fn finish_walk(&mut self, maze: &mut Maze) {
        for i in 0..self.walk.len()-1 {
            let start = self.walk[i];
            let end = self.walk[i + 1];

            let direction = {
                if start.0 == end.0 {
                    // Same X
                    if end.1 > start.1 {
                        // If end Y is more than start Y, down
                        Direction::DOWN
                    } else {
                        // Else, up
                        Direction::UP
                    }
                } else {
                    // Same Y
                    if end.0 > start.0 {
                        // If end X is more than start X, right
                        Direction::RIGHT
                    } else {
                        // Else, left
                        Direction::LEFT
                    }
                }
            };

            maze.carve(start.0, start.1, direction);
        }
    }

    /// Returns `true` if no possible starting positions can be made
    fn create_new_walk(&mut self, maze: &Maze) -> bool {
        self.walk.clear();

        let mut possible_next: Vec<(usize, usize)> = vec![];
        for (i, cell) in maze.cells.iter().enumerate() {
            if *cell == 0 {
                possible_next.push(maze.i_to_xy(i));
            }
        }

        if possible_next.len() == 0 {
            return true
        }

        self.walk.clear();
        self.walk.push(possible_next[rng().random_range(0..possible_next.len())]);
        self.opposite_of_last_direction = None;

        false
    }
}

impl Drawable for Wilson {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let mut polyline = VertexBuffer::new(PrimitiveType::LINE_STRIP, self.walk.len(), VertexBufferUsage::DYNAMIC).unwrap();

        let points: Vec<Vertex> = self.walk.iter().map(
            |(x, y)| Vertex::with_pos_color((((*x * 2 + 1) * CELL_SIZE / 2) as f32, ((*y * 2 + 1) * CELL_SIZE / 2) as f32).into(), Color::RED)
        ).collect();

        polyline.update(&points, 0).unwrap();

        target.draw_vertex_buffer(&polyline, rs);

        if let Some(pos) = self.first_walk_target {
            let radius = CELL_SIZE as f32 / 2.;

            let mut circle = CircleShape::new(
                radius,
                12
            );
            circle.set_fill_color(Color::GREEN);
            circle.set_origin((radius, radius));
            circle.set_position((
                (pos.0 * 2 + 1) as f32 * radius,
                (pos.1 * 2 + 1) as f32 * radius,
            ));

            target.draw(&circle);
        }
    }
}

impl MazeGenerator for Wilson {
    fn step(&mut self, maze: &mut Maze) -> bool {
        let pos = self.walk.last();

        match pos {
            None => return true,
            _ => {}
        }

        let pos = *pos.unwrap();

        let neighbors = maze.get_neighbors(pos);

        let next = loop {
            let next = neighbors[rng().random_range(0..neighbors.len())];

            if let Some(v) = self.opposite_of_last_direction {
                if v == next.2 {
                    continue
                }
            }

            break next
        };

        if let Some(break_index) = self.pos_in_stack((next.0, next.1)) {
            self.walk.drain(break_index+1..);
            return false
        }

        self.opposite_of_last_direction = Some(next.2.opposite());
        self.walk.push((next.0, next.1));
        self.current_walk_steps += 1;

        if  maze.get(next.0, next.1) != 0 {
            self.first_walk_target = None;
            self.finish_walk(maze);
            return self.create_new_walk(maze);
        } else if let Some(target) = self.first_walk_target {
            if target.0 == next.0 && target.1 == next.1 {
                self.first_walk_target = None;
                self.finish_walk(maze);
                return self.create_new_walk(maze);
            }
        }

        false
    }
}
