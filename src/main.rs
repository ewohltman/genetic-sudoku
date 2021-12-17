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
    genetics::{
        generate_initial_population,
        run_simulation,
        GAParams,
        MAX_POPULATION,
    },
    sudoku::Board,
};
use std::time::Instant;

fn main() {
    let matches = App::new("genetic-sudoku")
        .arg(
            Arg::with_name("population")
                .help("population per generation")
                .long("population")
                .takes_value(true)
                .value_name("N"),
        )
        .arg(
            Arg::with_name("selection")
                .help("fraction of population selected")
                .long("fraction")
                .takes_value(true)
                .value_name("S"),
        )
        .arg(
            Arg::with_name("mutation")
                .help("mutation rate as fraction")
                .long("mutation")
                .takes_value(true)
                .value_name("F"),
        )
        .arg(
            Arg::with_name("BOARD")
                .help("board to solve")
                .required(true),
        )
        .get_matches();
    let path = Path::new(matches.value_of("BOARD").unwrap());

    let population = matches
        .value_of("population")
        .unwrap_or("100")
        .parse()
        .unwrap();
    let selection = matches
        .value_of("selection")
        .unwrap_or("0.5")
        .parse()
        .unwrap();
    let mutation = matches
        .value_of("mutation")
        .unwrap_or("0.05")
        .parse()
        .unwrap();
    let params = GAParams::new(population, selection, mutation);

    let start = Instant::now();
    let mut runs: u32 = 0;
    let mut total_generations: u64 = 0;

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
