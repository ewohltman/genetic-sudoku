#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use genetic_sudoku::{genetics, sudoku, sudoku::Board};
use std::time::Instant;

const BASE: Board<9> = sudoku::al_escargot();

fn main() {
    let start = Instant::now();
    let mut runs: u32 = 0;
    let mut total_generations: u64 = 0;

    loop {
        runs += 1;

        let now = Instant::now();
        let mut generation: u64 = 0;
        let mut population =
            genetics::generate_initial_population::<{ BASE.size() }, { genetics::NUM_POPULATION }>(
            );

        loop {
            population = match genetics::run_simulation(&BASE, population) {
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
