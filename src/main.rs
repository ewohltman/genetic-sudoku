#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use clap::{App, Arg};
use std::path::Path;

use genetic_sudoku::{
    genetics::{generate_initial_population, run_simulation, GAParams, MAX_POPULATION},
    sudoku::Board,
};
use std::time::Instant;

fn main() {
    let matches = App::new("genetic-sudoku")
        .arg(
            Arg::with_name("BOARD")
                .help("board to solve")
                .required(true),
        )
        .get_matches();
    let path = Path::new(matches.value_of("BOARD").unwrap());

    let start = Instant::now();
    let mut runs: u32 = 0;
    let mut total_generations: u64 = 0;
    let params = GAParams::default();

    let board = Board::read(path).unwrap();

    loop {
        runs += 1;

        let now = Instant::now();
        let mut generation: u64 = 0;
        let mut population = generate_initial_population::<9, MAX_POPULATION>(&params);

        loop {
            population = match run_simulation(&params, &board, population) {
                Ok(_) => {
                    total_generations += generation;

                    println!(
                        "Solution: Generation: {} | Duration: {:?} | Average Generation: {} | Average Duration: {:?}",
                        generation,
                        now.elapsed(),
                        total_generations / u64::from(runs),
                        start.elapsed() / runs
                    );

                    break;
                }
                Err(no_solution_found) => {
                    generation += 1;
                    no_solution_found.next_generation
                }
            };
        }
    }
}
