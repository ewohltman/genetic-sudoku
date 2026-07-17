use clap::Parser;
use genetic_sudoku::{genetics, sudoku};
use ratatui::crossterm::event::{self, Event};
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
    #[arg(
        short,
        long,
        default_value_t = 100,
        value_parser = parse_population,
        help = "Population per generation"
    )]
    population: usize,

    #[arg(
        short,
        long,
        default_value_t = 0.5,
        value_parser = parse_selection_rate,
        help = "Fraction of population selected"
    )]
    selection_rate: f32,

    #[arg(
        short,
        long,
        default_value_t = 0.06,
        value_parser = parse_mutation_rate,
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

/// Parses a population argument, requiring `2..=MAX_POPULATION`.
fn parse_population(s: &str) -> Result<usize, String> {
    let population: usize = s.parse().map_err(|err| format!("{err}"))?;

    if (2..=genetics::MAX_POPULATION).contains(&population) {
        Ok(population)
    } else {
        Err(format!("must be in 2..={}", genetics::MAX_POPULATION))
    }
}

/// Parses a selection rate argument, requiring `(0.0, 1.0]`.
fn parse_selection_rate(s: &str) -> Result<f32, String> {
    let rate: f32 = s.parse().map_err(|err| format!("{err}"))?;

    if rate > 0.0 && rate <= 1.0 {
        Ok(rate)
    } else {
        Err(String::from("must be in (0.0, 1.0]"))
    }
}

/// Parses a mutation rate argument, requiring `[0.0, 1.0]`.
fn parse_mutation_rate(s: &str) -> Result<f32, String> {
    let rate: f32 = s.parse().map_err(|err| format!("{err}"))?;

    if (0.0..=1.0).contains(&rate) {
        Ok(rate)
    } else {
        Err(String::from("must be in [0.0, 1.0]"))
    }
}

fn main() -> Result<()> {
    simple_eyre::install()?;

    let args = Args::parse();

    ratatui::run(|terminal| run(args, terminal))
}

#[inline]
fn run(args: Args, terminal: &mut DefaultTerminal) -> Result<()> {
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
    if event::poll(poll_duration)?
        && let Event::Key(key) = event::read()?
        && key.is_press()
    {
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

    Ok(false)
}

/// Returns a widget with the current state of the simulation to be rendered.
#[inline]
#[must_use]
fn widget<const N: usize>(start: Instant, generation: usize, board: &sudoku::Board<N>) -> Text<'_> {
    Text::raw(format!(
        "Duration: {:?}\nGeneration: {generation}\nScore: {}\n\n{board}",
        start.elapsed(),
        board.fitness(),
    ))
}
