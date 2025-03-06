#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use clap::Parser;
use crossterm::event::{self, Event};
use genetic_sudoku::{genetics, sudoku};
use ratatui::{DefaultTerminal, Frame, text::Text};
use simple_eyre::Result;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

// The board size for puzzles. Change this for larger or smaller boards.
const BOARD_SIZE: usize = 9;

const POLL_DURATION: Duration = Duration::from_secs(1);

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

struct RenderUpdate {
    start: Instant,
    generation: usize,
    board: sudoku::Board<BOARD_SIZE>,
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_eyre::install()?;

    let result = run(Args::parse(), ratatui::init()).await;

    ratatui::restore();

    result
}

#[inline]
async fn run(args: Args, terminal: DefaultTerminal) -> Result<()> {
    let start = Instant::now();
    let board = sudoku::Board::read(args.board_path)?;
    let params = genetics::GAParams::new(args.population, args.selection_rate, args.mutation_rate);
    let render_generations = args.render.unwrap_or(1);

    let mut generation: usize = 0;
    let mut population = genetics::generate_initial_population::<BOARD_SIZE>(&params);

    let (quit_tx, mut quit_rx) = mpsc::channel::<()>(1);
    let (render_tx, render_rx) = mpsc::channel::<RenderUpdate>(1);

    watch_quit(quit_tx.clone());
    render(terminal, quit_tx.clone(), render_rx);

    tokio::spawn(async move {
        loop {
            if quit_tx.is_closed() {
                return;
            }

            population = match genetics::run_simulation::<BOARD_SIZE>(&params, &board, population) {
                Ok(solution) => {
                    let _ = render_tx
                        .send(RenderUpdate {
                            start,
                            generation,
                            board: solution,
                        })
                        .await;

                    return;
                }
                Err(no_solution_found) => {
                    if generation == 0 || generation % render_generations == 0 {
                        let _ = render_tx
                            .send(RenderUpdate {
                                start,
                                generation,
                                board: no_solution_found.next_generation[0],
                            })
                            .await;
                    }

                    generation += 1;

                    no_solution_found.next_generation
                }
            };
        }
    });

    let _ = quit_rx.recv().await;

    Ok(())
}

#[inline]
fn watch_quit(quit_tx: Sender<()>) {
    tokio::spawn(async move {
        loop {
            if quit_tx.is_closed() {
                return;
            }

            match should_quit() {
                Ok(quit) => {
                    if quit {
                        let _ = quit_tx.send(()).await;
                    }
                }
                Err(_) => {
                    let _ = quit_tx.send(()).await;
                }
            }
        }
    });
}

#[inline]
fn render(
    mut terminal: DefaultTerminal,
    quit_tx: Sender<()>,
    mut render_rx: Receiver<RenderUpdate>,
) {
    tokio::spawn(async move {
        loop {
            if quit_tx.is_closed() {
                return;
            }

            match render_rx.recv().await {
                Some(update) => {
                    let _ = terminal.draw(|frame: &mut Frame| {
                        frame.render_widget(
                            widget(update.start, update.generation, &update.board),
                            frame.area(),
                        );
                    });
                }
                None => return,
            }
        }
    });
}

/// Returns whether we should quit because Ctrl+c, q, or escape was pressed.
#[inline]
fn should_quit() -> Result<bool> {
    if event::poll(POLL_DURATION)? {
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
