use std::collections::{HashMap, HashSet, VecDeque};

use sfml::graphics::{Color, Drawable, PrimitiveType, RectangleShape, Shape, Transformable, Vertex, VertexBuffer, VertexBufferUsage};

use crate::maze::{Maze, MazeSolver};
use crate::consts::*;

pub struct DepthFirstSearch {
    visited: HashSet<(usize, usize)>,
    path: Vec<(usize, usize)>,

    end: (usize, usize),
}

impl DepthFirstSearch {
    pub fn new(bounds: (usize, usize)) -> Self {
        return Self{
            visited: HashSet::new(),
            path: vec![(0, 0)],

            end: (bounds.0 - 1, bounds.1 - 1)
        }
    }
}

impl Drawable for DepthFirstSearch {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let mut polyline = VertexBuffer::new(PrimitiveType::LINE_STRIP, self.path.len(), VertexBufferUsage::DYNAMIC).unwrap();

        let points: Vec<Vertex> = self.path.iter().map(
            |(x, y)| Vertex::with_pos_color((((*x * 2 + 1) * CELL_SIZE / 2) as f32, ((*y * 2 + 1) * CELL_SIZE / 2) as f32).into(), Color::RED)
        ).collect();

        polyline.update(&points, 0).unwrap();

        target.draw_vertex_buffer(&polyline, rs);
    }
}

impl MazeSolver for DepthFirstSearch {
    fn step(&mut self, maze: &Maze) -> Option<&Vec<(usize, usize)>> {
        let pos = *self.path.last().unwrap();

        if pos == self.end {
            return Some(&self.path);
        }

        let neighbors = maze.get_travellable_neighbors(pos);
        let next = neighbors.iter().filter_map(
            |v| if self.visited.contains(&(v.0, v.1)) {
                None
            } else {
                Some((v.0, v.1))
            }
        ).nth(0);

        self.visited.insert(pos);
        match next {
            Some(v) => {
                self.path.push(v)
            },
            None => { self.path.pop(); },
        }

        None
    }
}

pub struct BreadthFirstSearch {
    queue: VecDeque<(usize, usize)>,
    visited: HashMap<(usize, usize), Option<(usize, usize)>>,

    path: Vec<(usize, usize)>,
    finished: bool,

    end: (usize, usize)
}

impl BreadthFirstSearch {
    pub fn new(bounds: (usize, usize)) -> Self {
        let mut queue = VecDeque::new();
        let mut visited: HashMap<(usize, usize), Option<(usize, usize)>> = HashMap::new();

        queue.push_back((0, 0));
        visited.insert((0, 0), None);

        return Self{
            visited,
            queue,

            path: vec![],
            finished: false,

            end: (bounds.0 - 1, bounds.1 - 1)
        }
    }
}

impl Drawable for BreadthFirstSearch {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        _: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let mut rect = RectangleShape::with_size(
            (CELL_SIZE as f32 / 2.,
            CELL_SIZE as f32 / 2.).into()
        );
        rect.set_origin((CELL_SIZE as f32 / 4., CELL_SIZE as f32 / 4.));

        for pos in self.visited.keys() {
            rect.set_fill_color(Color::rgba(0, 255, 0, if self.finished {
                if self.path.contains(pos) {
                    255
                } else {
                    64
                }
            } else {
                255
            }));

            rect.set_position((
                ((pos.0 * 2 + 1) * CELL_SIZE / 2) as f32,
                ((pos.1 * 2 + 1) * CELL_SIZE / 2) as f32,
            ));
    
            target.draw(&rect);
        }
    }
}

impl MazeSolver for BreadthFirstSearch {
    fn step(&mut self, maze: &Maze) -> Option<&Vec<(usize, usize)>> {
        if self.finished {
            return Some(&self.path);
        }
        
        let pos = self.queue.pop_front().unwrap();

        if pos == self.end {
            self.finished = true;

            let mut pos = pos;

            loop {
                self.path.push(pos);

                let prev = self.visited.get(&pos).unwrap();

                match prev {
                    None => break,
                    Some(v) => pos = *v,
                }
            }

            self.path.reverse();

            return Some(&self.path)
        }

        let neighbors = maze.get_travellable_neighbors(pos);
        let next: Vec<_> = neighbors.iter().filter_map(
            |v| if self.visited.contains_key(&(v.0, v.1)) {
                None
            } else {
                Some((v.0, v.1))
            }
        ).collect();

        for next_pos in next {
            self.visited.insert(next_pos, Some(pos));
            self.queue.push_back(next_pos);
        }

        None
    }
}
