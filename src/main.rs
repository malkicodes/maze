use std::fs;

use maze::maze::{generators::*, MazeSolver};
use maze::consts::*;
use maze::maze::solvers::{BreadthFirstSearch, DepthFirstSearch};
use maze::maze::{Maze, MazeGenerator};
use sfml::{
    graphics::{Color, RenderTarget, RenderWindow},
    window::{Event, Style},
};

const DEBUG: bool = false;
const DEBUG_STEPS_PER_SECOND: Option<u32> = None;

fn main() {
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

    if DEBUG {
        match DEBUG_STEPS_PER_SECOND {
            Some(v) => window.set_framerate_limit(v),
            None => window.set_vertical_sync_enabled(true),
        }
    } else {
        window.set_vertical_sync_enabled(true)
    }

    let mut maze = Maze::new(MAZE_WIDTH, MAZE_HEIGHT);
    let mut generator = Wilson::new(maze.get_bounds());
    let mut solver = DepthFirstSearch::new(maze.get_bounds());

    let mut generated = false;

    if !DEBUG {
        maze.generate(&mut generator);
        generated = true
    }

    'mainloop: loop {
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => break 'mainloop,
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
