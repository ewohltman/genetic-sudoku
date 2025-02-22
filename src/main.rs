#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use clap::Parser;
use genetic_sudoku::{
    genetics::{GAParams, MAX_POPULATION, generate_initial_population, run_simulation},
    sudoku::Board,
};
use std::error::Error;
use std::ops::Add;
use std::path::PathBuf;
use std::time::{Duration, Instant};

// The board size for puzzles. Change this for larger or smaller boards.
const BOARD_SIZE: usize = 9;

const MAX_GENERATIONS: usize = 100_000;

const TIMEOUT: u64 = 600;

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
        default_value_t = 0.05,
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
    let params = GAParams::new(
        args.population,
        args.selection_rate,
        args.mutation_rate,
        args.restart,
    );
    let board = Board::read(args.board_path)?;

    let start = Instant::now();
    let timeout = start.add(Duration::new(TIMEOUT, 0));
    let mut runs: usize = 0;
    let mut total_generations: usize = 0;

    loop {
        runs += 1;

        let now = Instant::now();
        let mut generation: usize = 0;
        let mut population = generate_initial_population::<BOARD_SIZE, MAX_POPULATION>(&params);

        loop {
            population = match run_simulation::<BOARD_SIZE, MAX_POPULATION>(
                &params, generation, &board, population,
            ) {
                Ok(solution) => {
                    total_generations += generation;

                    print!("Generation: {generation} | Duration: {:?}", now.elapsed(),);

                    if args.benchmark {
                        println!(
                            " | Average Generation: {} | Average Duration: {:?}",
                            total_generations / runs,
                            start.elapsed() / u32::try_from(runs)?
                        );

                        break;
                    }

                    println!("\n{solution}");

                    return Ok(());
                }
                Err(no_solution_found) => {
                    generation += 1;

                    if generation >= MAX_GENERATIONS || Instant::now().ge(&timeout) {
                        println!("Generation: {generation} | Duration: {:?}", now.elapsed());

                        return Err(no_solution_found.into());
                    }

                    no_solution_found.next_generation
                }
            };
        }
    }
}
