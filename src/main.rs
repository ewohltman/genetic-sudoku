#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use clap::Parser;
use genetic_sudoku::{genetics, sudoku};
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;

// The board size for puzzles. Change this for larger or smaller boards.
const BOARD_SIZE: usize = 9;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 100, help = "Population per generation")]
    population: usize,

    #[arg(
        short,
        long,
        default_value_t = 0.5,
        help = "Fraction of population selected"
    )]
    selection_rate: f32,

    #[arg(
        short,
        long,
        default_value_t = 0.06,
        help = "Mutation rate as fraction"
    )]
    mutation_rate: f32,

    #[arg(short, long, help = "Number of generations to restart population")]
    restart: Option<usize>,

    #[arg(short, long, help = "Run program in benchmark mode")]
    benchmark: bool,

    #[arg(help = "Path to board file")]
    board_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let params = genetics::GAParams::new(
        args.population,
        args.selection_rate,
        args.mutation_rate,
        args.restart,
    );
    let board = sudoku::Board::read(args.board_path)?;

    let start = Instant::now();
    let mut runs: usize = 0;
    let mut total_generations: usize = 0;

    loop {
        runs += 1;

        let now = Instant::now();
        let mut generation: usize = 0;
        let mut population = genetics::generate_initial_population::<BOARD_SIZE>(&params);

        loop {
            population = match genetics::run_simulation::<BOARD_SIZE>(
                &params, generation, &board, population,
            ) {
                Ok(solution) => {
                    total_generations += generation;

                    print!("Generation: {generation} | Duration: {:?}", now.elapsed());

                    if args.benchmark {
                        println!(
                            " | Average Generation: {} | Average Duration: {:?}",
                            total_generations / runs,
                            start.elapsed() / u32::try_from(runs)?
                        );

                        break;
                    }

                    println!("\n{solution}\nSolution found");

                    return Ok(());
                }
                Err(no_solution_found) => {
                    if generation % 1000 == 0 {
                        let best_board = no_solution_found.next_generation[0];

                        println!(
                            "Generation: {generation} | Duration: {:?} | Score: {}\n{best_board}",
                            now.elapsed(),
                            best_board.fitness(),
                        );
                    }

                    generation += 1;

                    no_solution_found.next_generation
                }
            };
        }
    }
}
