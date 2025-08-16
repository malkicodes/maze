use std::fs;
use std::time::Instant;

use clap::{Parser, ValueEnum};
use maze::maze::{generators::*, solvers::*, MazeSolver};
use maze::consts::*;
use maze::maze::{Maze, MazeGenerator};
use sfml::window::{Key, VideoMode};
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
#[command(version, about, long_about = None, disable_help_flag = true)]
struct Cli {
    /// Input maze data path
    #[arg(short, long)]
    input: Option<String>,

    /// Output maze data path
    #[arg(short, long)]
    output: Option<String>,

    /// See generation live
    #[arg(short, long)]
    debug: bool,

    /// Which algorithm to use
    #[arg(short, long, default_value_t = AlgorithmArg::DFS)]
    alg: AlgorithmArg,

    /// Maze width
    #[arg(short, long, default_value_t = DEFAULT_MAZE_WIDTH)]
    width: u16,

    /// Maze height
    #[arg(short, long, default_value_t = DEFAULT_MAZE_HEIGHT)]
    height: u16,

    /// Instantly solve the maze
    #[arg(long, default_value_t = false)]
    instant: bool,

    /// Display help
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

fn main() {
    let cli: Cli = Cli::parse();
    
    let mut generated = false;
    let mut maze = match &cli.input {
        None => {
            Maze::new(cli.width, cli.height)
        },
        Some(path) => {
            let data = fs::read(path).unwrap();
            
            generated = true;
            let maze = Maze::from_data(&data).unwrap();

            maze
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
        let mut step_count: usize = 0;
        
        let start = Instant::now();
        while !generator.step(&mut maze) {
            step_count += 1;
        }
        let duration = start.elapsed();
        
        println!("Generating maze took {} steps and {:?}", step_count, duration);
        
        generated = true
    }

    if let Some(path) = &cli.output {
        match fs::write(path, maze.as_str().unwrap()) {
            Ok(_) => println!("Wrote maze data to {path}"),
            Err(err) => println!("Could not write to file: {err}")
        };
    }

    let mut window = RenderWindow::new(
        {
            let (maze_width, maze_height) = maze.get_bounds();

            VideoMode::new((maze_width * CELL_SIZE) as u32, (maze_height * CELL_SIZE) as u32, 32)
        },
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

    if cli.instant {
        let mut step_count: usize = 0;
        
        let start = Instant::now();
        while let None = solver.step(&maze) {
            step_count += 1;
        }
        let duration = start.elapsed();

        println!("Solving maze took {step_count} steps and {duration:?}")
    }

    'mainloop: loop {
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => break 'mainloop,
                Event::KeyPressed { code, ctrl, .. } => if code == Key::Q || (code == Key::C && ctrl) {
                    break 'mainloop;
                },
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
}
