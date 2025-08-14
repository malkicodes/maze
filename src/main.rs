use std::fs;

use clap::{Parser, ValueEnum};
use maze::maze::{generators::*, MazeSolver};
use maze::consts::*;
use maze::maze::solvers::{AStarSolver, Algorithm, BFSSolver, DFSSolver};
use maze::maze::{Maze, MazeGenerator};
use sfml::window::Key;
use sfml::{
    graphics::{Color, RenderTarget, RenderWindow},
    window::{Event, Style},
};

const DEBUG_STEPS_PER_SECOND: u32 = 144 * 2;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum AlgorithmArg {
    /// Depth-First Search
    DFS,
    /// Breadth-First Search
    BFS,
    AStar,
}

impl ToString for AlgorithmArg {
    fn to_string(&self) -> String {
        match self {
            AlgorithmArg::BFS => String::from("bfs"),
            AlgorithmArg::DFS => String::from("dfs"),
            AlgorithmArg::AStar => String::from("a-star"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Maze data path
    #[arg(short, long)]
    maze: Option<String>,

    /// See generation live
    #[arg(short, long)]
    debug: bool,

    /// Which algorithm to use
    #[arg(long, default_value_t = AlgorithmArg::DFS)]
    alg: AlgorithmArg,
}

fn main() {
    let cli: Cli = Cli::parse();

    let mut window = RenderWindow::new(
        (
            (MAZE_WIDTH * CELL_SIZE) as u32,
            (MAZE_HEIGHT * CELL_SIZE) as u32,
        ),
        "Maze",
        Style::CLOSE,
        &Default::default(),
    )
    .unwrap();

    if cli.debug {
        window.set_framerate_limit(DEBUG_STEPS_PER_SECOND);
    } else {
        window.set_vertical_sync_enabled(true)
    }
    
    let mut generated = false;
    let mut maze = match cli.maze {
        None => {
            Maze::new(MAZE_WIDTH, MAZE_HEIGHT)
        },
        Some(path) => {
            let data = String::from_utf8(fs::read(path).unwrap()).unwrap();
            
            generated = true;
            Maze::from_str(&data).unwrap()
        }
    };
    let mut generator = Wilson::new(maze.get_bounds());

    let bounds = maze.get_bounds();

    let mut solver: Algorithm = match cli.alg {
        AlgorithmArg::BFS => Algorithm::BreadthFirstSearch(BFSSolver::new(bounds)),
        AlgorithmArg::DFS => Algorithm::DepthFirstSearch(DFSSolver::new(bounds)),
        AlgorithmArg::AStar => Algorithm::AStar(AStarSolver::new(bounds))
    };

    if !(generated || cli.debug) {
        maze.generate(&mut generator);
        generated = true
    }

    'mainloop: loop {
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => break 'mainloop,
                Event::KeyPressed { code: Key::Q, .. } => break 'mainloop,
                _ => {}
            }
        }

        if !generated {
            generated = generator.step(&mut maze);
        } else {
            solver.step(&maze);
        }

        window.clear(Color::BLACK);

        window.draw(&maze);

        if !generated {
            window.draw(&generator);
        } else {
            window.draw(&solver);
        }

        window.display();
    }

    match fs::write("maze.dat", maze.as_str()) {
        Ok(_) => println!("Wrote maze data to maze.dat"),
        Err(err) => println!("Could not write to file: {}", err)
    };
}
