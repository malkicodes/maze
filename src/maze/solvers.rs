use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use sfml::graphics::{
    Color, Drawable, PrimitiveType, RectangleShape, Shape, Transformable, Vertex, VertexBuffer,
    VertexBufferUsage,
};

use crate::consts::*;
use crate::maze::{Maze, MazeSolver};

pub enum Algorithm {
    DepthFirstSearch(DFSSolver),
    BreadthFirstSearch(BFSSolver),
    AStar(AStarSolver),
}

impl Algorithm {
    pub fn step(&mut self, maze: &Maze) -> Option<&Vec<(usize, usize)>> {
        match self {
            Self::BreadthFirstSearch(v) => v.step(maze),
            Self::DepthFirstSearch(v) => v.step(maze),
            Self::AStar(v) => v.step(maze),
        }
    }
}

impl Drawable for Algorithm {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        match self {
            Self::BreadthFirstSearch(v) => v.draw(target, rs),
            Self::DepthFirstSearch(v) => v.draw(target, rs),
            Self::AStar(v) => v.draw(target, rs),
        };
    }
}

pub struct DFSSolver {
    visited: HashSet<(usize, usize)>,
    path: Vec<(usize, usize)>,

    end: (usize, usize),
}

impl MazeSolver for DFSSolver {
    fn new(bounds: (usize, usize)) -> Self {
        Self {
            visited: HashSet::new(),
            path: vec![(0, 0)],

            end: (bounds.0 - 1, bounds.1 - 1),
        }
    }

    fn step(&mut self, maze: &Maze) -> Option<&Vec<(usize, usize)>> {
        let pos = *self.path.last().unwrap();

        if pos == self.end {
            return Some(&self.path);
        }

        let neighbors = maze.get_travellable_neighbors(pos);
        let next = (0..neighbors.1)
            .filter_map(|i| {
                if self.visited.contains(&neighbors.0[i]) {
                    None
                } else {
                    Some(neighbors.0[i])
                }
            })
            .nth(0);

        self.visited.insert(pos);
        match next {
            Some(v) => self.path.push(v),
            None => {
                self.path.pop();
            }
        }

        None
    }
}

pub struct BFSSolver {
    queue: VecDeque<(usize, usize)>,
    visited: HashMap<(usize, usize), Option<(usize, usize)>>,

    path: Vec<(usize, usize)>,
    finished: bool,

    end: (usize, usize),
}

impl MazeSolver for BFSSolver {
    fn new(bounds: (usize, usize)) -> Self {
        let mut queue = VecDeque::new();
        let mut visited: HashMap<(usize, usize), Option<(usize, usize)>> = HashMap::new();

        queue.push_back((0, 0));
        visited.insert((0, 0), None);

        Self {
            visited,
            queue,

            path: vec![],
            finished: false,

            end: (bounds.0 - 1, bounds.1 - 1),
        }
    }

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

            return Some(&self.path);
        }

        let neighbors = maze.get_travellable_neighbors(pos);
        let next: Vec<_> = (0..neighbors.1)
            .filter_map(|i| {
                if self.visited.contains_key(&neighbors.0[i]) {
                    None
                } else {
                    Some(neighbors.0[i])
                }
            })
            .collect();

        for next_pos in next {
            self.visited.insert(next_pos, Some(pos));
            self.queue.push_back(next_pos);
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CellInformation {
    f_cost: usize,
    h_cost: usize,
    from: Option<(usize, usize)>,
}

pub struct AStarSolver {
    open: BTreeMap<(usize, usize), CellInformation>,
    closed: HashMap<(usize, usize), CellInformation>,

    end: (usize, usize),

    path: Vec<(usize, usize)>,
}

impl MazeSolver for AStarSolver {
    fn new(bounds: (usize, usize)) -> Self {
        let mut open = BTreeMap::new();

        open.insert(
            (0, 0),
            CellInformation {
                h_cost: 0,
                f_cost: bounds.0 + bounds.1,
                from: None,
            },
        );

        Self {
            open,
            closed: HashMap::new(),

            end: (bounds.0 - 1, bounds.1 - 1),

            path: Vec::new(),
        }
    }

    fn step(&mut self, maze: &Maze) -> Option<&Vec<(usize, usize)>> {
        if !self.path.is_empty() {
            return Some(&self.path);
        }

        let mut current_data: (&(usize, usize), &CellInformation) = (
            &(1, 1),
            &CellInformation {
                from: None,
                f_cost: usize::MAX,
                h_cost: usize::MAX,
            },
        );

        for (pos, info) in self.open.iter() {
            if info.f_cost < current_data.1.f_cost
                || (info.f_cost == current_data.1.f_cost && info.h_cost <= current_data.1.h_cost)
            {
                current_data.0 = pos;
                current_data.1 = info;
            }
        }

        let current_pos = *current_data.0;
        let current = *current_data.1;

        self.closed.insert(current_pos, current);
        self.open.remove(&current_pos);

        if current_pos == self.end {
            let mut pos = current_pos;

            loop {
                self.path.push(pos);

                let prev = self.closed.get(&pos).unwrap();

                match prev.from {
                    None => break,
                    Some(v) => pos = v,
                }
            }

            self.path.reverse();

            return Some(&self.path);
        }

        let neighbors = maze.get_travellable_neighbors(current_pos);

        for neighbor in (0..neighbors.1).map(|i| neighbors.0[i]) {
            if self.closed.contains_key(&neighbor) {
                continue;
            }

            let g_cost = current.h_cost + 1;
            let h_cost = current_pos.0.abs_diff(self.end.0) + current_pos.1.abs_diff(self.end.1);
            let f_cost = g_cost + h_cost;

            if if let Some(v) = self.closed.get(&neighbor) {
                h_cost < v.h_cost
            } else {
                false
            } {
                self.closed.insert(
                    neighbor,
                    CellInformation {
                        f_cost,
                        h_cost,
                        from: Some(current_pos),
                    },
                );
                continue;
            }

            self.open.entry(neighbor).or_insert(CellInformation {
                f_cost,
                h_cost,
                from: Some(current_pos),
            });
        }

        None
    }
}

impl Drawable for DFSSolver {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let cell_size = get_cell_size();

        let mut polyline = VertexBuffer::new(
            PrimitiveType::LINE_STRIP,
            self.path.len(),
            VertexBufferUsage::DYNAMIC,
        )
        .unwrap();

        let points: Vec<Vertex> = self
            .path
            .iter()
            .map(|(x, y)| {
                Vertex::with_pos_color(
                    (
                        ((*x * 2 + 1) * cell_size / 2) as f32,
                        ((*y * 2 + 1) * cell_size / 2) as f32,
                    )
                        .into(),
                    Color::RED,
                )
            })
            .collect();

        polyline.update(&points, 0).unwrap();

        target.draw_vertex_buffer(&polyline, rs);
    }
}

impl Drawable for BFSSolver {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let cell_size = get_cell_size();

        let mut rect =
            RectangleShape::with_size((cell_size as f32 / 2., cell_size as f32 / 2.).into());
        rect.set_origin((cell_size as f32 / 4., cell_size as f32 / 4.));

        for pos in self.visited.keys() {
            rect.set_fill_color(Color::rgba(
                0,
                255,
                0,
                if self.finished {
                    if self.path.contains(pos) {
                        255
                    } else {
                        64
                    }
                } else {
                    255
                },
            ));

            rect.set_position((
                ((pos.0 * 2 + 1) * cell_size / 2) as f32,
                ((pos.1 * 2 + 1) * cell_size / 2) as f32,
            ));

            target.draw(&rect);
        }

        let mut polyline = VertexBuffer::new(
            PrimitiveType::LINE_STRIP,
            self.path.len(),
            VertexBufferUsage::DYNAMIC,
        )
        .unwrap();

        let points: Vec<Vertex> = self
            .path
            .iter()
            .map(|(x, y)| {
                Vertex::with_pos_color(
                    (
                        ((*x * 2 + 1) * cell_size / 2) as f32,
                        ((*y * 2 + 1) * cell_size / 2) as f32,
                    )
                        .into(),
                    Color::RED,
                )
            })
            .collect();

        polyline.update(&points, 0).unwrap();

        target.draw_vertex_buffer(&polyline, rs);
    }
}

impl Drawable for AStarSolver {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let cell_size = get_cell_size();

        let finished = !self.path.is_empty();

        let mut rect =
            RectangleShape::with_size((cell_size as f32 / 2., cell_size as f32 / 2.).into());
        rect.set_origin((cell_size as f32 / 4., cell_size as f32 / 4.));

        for pos in self.closed.keys() {
            rect.set_fill_color(Color::rgba(
                255,
                0,
                0,
                if finished {
                    if self.path.contains(pos) {
                        255
                    } else {
                        64
                    }
                } else {
                    255
                },
            ));
            rect.set_position((
                ((pos.0 * 2 + 1) * cell_size / 2) as f32,
                ((pos.1 * 2 + 1) * cell_size / 2) as f32,
            ));

            target.draw(&rect);
        }

        rect.set_fill_color(Color::rgba(0, 255, 0, if finished { 64 } else { 255 }));

        for pos in self.open.keys() {
            rect.set_position((
                ((pos.0 * 2 + 1) * cell_size / 2) as f32,
                ((pos.1 * 2 + 1) * cell_size / 2) as f32,
            ));

            target.draw(&rect);
        }

        if !self.path.is_empty() {
            let mut polyline = VertexBuffer::new(
                PrimitiveType::LINE_STRIP,
                self.path.len(),
                VertexBufferUsage::DYNAMIC,
            )
            .unwrap();

            let points: Vec<Vertex> = self
                .path
                .iter()
                .map(|(x, y)| {
                    Vertex::with_pos_color(
                        (
                            ((*x * 2 + 1) * cell_size / 2) as f32,
                            ((*y * 2 + 1) * cell_size / 2) as f32,
                        )
                            .into(),
                        Color::RED,
                    )
                })
                .collect();

            polyline.update(&points, 0).unwrap();

            target.draw_vertex_buffer(&polyline, rs);
        }
    }
}
