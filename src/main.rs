#![warn(
clippy::all,
// clippy::restriction,
clippy::pedantic,
clippy::nursery,
clippy::cargo,
)]

use genetic_sudoku::{genetics, sudoku::Board};
use std::error::Error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let base = Board::default();
    let mut runs: u32 = 0;
    let mut total_generations: u64 = 0;

    loop {
        runs += 1;

        let now = Instant::now();
        let mut generation: u64 = 0;
        let mut candidates = genetics::generate_initial_boards();

        loop {
            candidates = match genetics::run_simulation(&base, candidates) {
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
                    no_solution_found.next
                }
            };
        }
    }
}
