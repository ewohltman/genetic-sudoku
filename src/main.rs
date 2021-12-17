#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use clap::{App, Arg};
use genetic_sudoku::{
    genetics::{generate_initial_population, run_simulation, GAParams, MAX_POPULATION},
    sudoku::Board,
};
use std::path::{Path, PathBuf};
use std::time::Instant;

// The board size for puzzles. Change this for larger or smaller boards.
const BOARD_SIZE: usize = 9;

fn parse_args() -> Result<(PathBuf, GAParams, bool), Box<dyn std::error::Error>> {
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
            Arg::with_name("restart")
                .help("number of generations to restart population")
                .long("restart")
                .takes_value(true)
                .value_name("R"),
        )
        .arg(
            Arg::with_name("bench")
                .help("runs program in benchmark mode")
                .long("bench")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("BOARD")
                .help("board to solve")
                .required(true),
        )
        .get_matches();

    let path = Path::new(matches.value_of("BOARD").unwrap()).to_owned();
    let population = matches.value_of("population").unwrap_or("100").parse()?;
    let selection = matches.value_of("selection").unwrap_or("0.5").parse()?;
    let mutation = matches.value_of("mutation").unwrap_or("0.05").parse()?;
    let restart = match matches.value_of("restart") {
        None => None,
        Some(restart) => Some(restart.parse()?),
    };
    let benchmark = matches.is_present("bench");
    let params = GAParams::new(population, selection, mutation, restart);

    Ok((path, params, benchmark))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (path, params, benchmark) = parse_args()?;
    let board = Board::read(path)?;

    let start = Instant::now();
    let mut runs: u32 = 0;
    let mut total_generations: u64 = 0;

    loop {
        runs += 1;

        let now = Instant::now();
        let mut generation: u64 = 0;
        let mut population = generate_initial_population::<BOARD_SIZE, MAX_POPULATION>(&params);

        loop {
            population = match run_simulation(&params, generation, &board, population) {
                Ok(solution) => {
                    total_generations += generation;

                    println!(
                        "Solution: Generation: {} | Duration: {:?} | Average Generation: {} | Average Duration: {:?}",
                        generation,
                        now.elapsed(),
                        total_generations / u64::from(runs),
                        start.elapsed() / runs
                    );

                    if !benchmark {
                        println!("{}", solution);
                        return Ok(());
                    }

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
