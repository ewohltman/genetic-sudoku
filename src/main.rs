#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use clap::Parser;
use crossterm::event::{self, Event};
use genetic_sudoku::{genetics, sudoku};
use ratatui::{DefaultTerminal, Frame, text::Text};
use simple_eyre::{Report, Result};
use std::path::PathBuf;
use std::time::{Duration, Instant};

// The board size for puzzles. Change this for larger or smaller boards.
const BOARD_SIZE: usize = 9;

const POLL_DURATION_RUNNING: Duration = Duration::from_nanos(1);

const POLL_DURATION_DONE: Duration = Duration::from_millis(100);

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

    #[arg(
        short,
        long,
        help = "Number of generations between screen renders. Higher values give better computational performance"
    )]
    render: Option<usize>,

    #[arg(help = "Path to board file")]
    board_path: PathBuf,
}

fn main() -> Result<()> {
    simple_eyre::install()?;

    let result = run(Args::parse(), ratatui::init());

    ratatui::restore();

    result
}

#[inline]
fn run(args: Args, mut terminal: DefaultTerminal) -> Result<()> {
    let start = Instant::now();
    let board = sudoku::Board::read(args.board_path)?;
    let params = genetics::GAParams::new(args.population, args.selection_rate, args.mutation_rate);
    let render = args.render.unwrap_or(1);
    let mut generation: usize = 0;
    let mut population = genetics::generate_initial_population::<BOARD_SIZE>(&params);

    loop {
        population = match genetics::run_simulation::<BOARD_SIZE>(&params, &board, population) {
            Ok(solution) => {
                terminal.draw(|frame: &mut Frame| {
                    frame.render_widget(widget(start, generation, &solution), frame.area());
                })?;

                loop {
                    if should_quit(POLL_DURATION_DONE)? {
                        return Ok(());
                    }
                }
            }
            Err(no_solution_found) => {
                if should_quit(POLL_DURATION_RUNNING)? {
                    return Err(Report::new(no_solution_found));
                }

                if generation % render == 0 {
                    let best_board = no_solution_found.next_generation[0];

                    terminal.draw(|frame: &mut Frame| {
                        frame.render_widget(widget(start, generation, &best_board), frame.area());
                    })?;
                }

                generation += 1;

                no_solution_found.next_generation
            }
        };
    }
}

/// Returns whether we should quit because Ctrl+c, q, or escape was pressed.
#[inline]
fn should_quit(poll_duration: Duration) -> Result<bool> {
    if event::poll(poll_duration)? {
        if let Event::Key(key) = event::read()? {
            let ctrl = key.modifiers.contains(event::KeyModifiers::CONTROL);
            let c = key.code == event::KeyCode::Char('c');

            if ctrl && c {
                return Ok(true);
            }

            return Ok(matches!(
                key.code,
                event::KeyCode::Char('q') | event::KeyCode::Esc
            ));
        }
    }

    Ok(false)
}

/// Returns a widget with the current state of the simulation to be rendered.
#[inline]
#[must_use]
fn widget<const N: usize>(start: Instant, generation: usize, board: &sudoku::Board<N>) -> Text {
    Text::raw(format!(
        "Duration: {:?}\nGeneration: {generation}\nScore: {}\n\n{board}",
        start.elapsed(),
        board.fitness(),
    ))
}
