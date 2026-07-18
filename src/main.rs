use clap::Parser;
use genetic_sudoku::errors::NoSolutionFound;
use genetic_sudoku::{genetics, sudoku};
use ratatui::crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame, text::Text};
use simple_eyre::{Report, Result};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, sync_channel};
use std::thread;
use std::time::{Duration, Instant};

// The board size for puzzles. Change this for larger or smaller boards.
const BOARD_SIZE: usize = 9;

// Poll timeout while the simulation is running. Doubles as the frame cadence
// (~30 fps).
const POLL_DURATION_RUNNING: Duration = Duration::from_millis(33);

const POLL_DURATION_DONE: Duration = Duration::from_millis(100);

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        default_value_t = 75,
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
        default_value_t = 0,
        help = "Number of top individuals carried unchanged into the next generation (0 = disabled)"
    )]
    elitism: usize,

    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "Greedy local-search passes applied to each child (0 = disabled)"
    )]
    local_search: usize,

    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "Number of generations without improvement before restarting with a new random population (0 = disabled)"
    )]
    restart: usize,

    #[arg(help = "Path to board file")]
    board_path: PathBuf,
}

/// A point-in-time view of the simulation, published by the simulation thread
/// for the render loop to draw.
#[derive(Clone, Copy)]
struct Snapshot {
    generation: usize,
    board: sudoku::Board<BOARD_SIZE>,
    score: u16,
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
    let params = genetics::GAParams::new(
        args.population,
        args.selection_rate,
        args.mutation_rate,
        args.elitism,
        args.local_search,
    );
    let quit = AtomicBool::new(false);
    let (tx, rx) = sync_channel::<Snapshot>(1);

    thread::scope(|scope| {
        // `tx` is moved into `simulate` so it drops when the simulation
        // thread exits, letting `render_loop` observe the disconnect.
        let simulation = scope.spawn(|| simulate(&params, &board, args.restart, &quit, tx));

        let render_result = render_loop(terminal, start, &rx);

        // Stop the simulation thread, then drop the receiver so a sender
        // blocked on a full channel unblocks instead of deadlocking the join.
        quit.store(true, Ordering::Relaxed);
        drop(rx);

        let simulation_result = simulation.join();

        render_result?;

        match simulation_result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(no_solution_found)) => Err(Report::new(no_solution_found)),
            Err(_) => Err(Report::msg("simulation thread panicked")),
        }
    })
}

/// Runs the genetic algorithm until a solution is found or `quit` is set.
///
/// Publishes a best-effort `Snapshot` of the best board every generation via
/// `tx`, dropping frames whenever the render loop has not yet consumed the
/// previous one. The final solved snapshot is always delivered.
// `tx` is taken by value so it drops when this function returns (or
// unwinds), disconnecting the channel for `render_loop`.
#[allow(clippy::needless_pass_by_value)]
fn simulate(
    params: &genetics::GAParams,
    board: &sudoku::Board<BOARD_SIZE>,
    restart: usize,
    quit: &AtomicBool,
    tx: SyncSender<Snapshot>,
) -> Result<(), NoSolutionFound<BOARD_SIZE>> {
    let mut generation: usize = 0;
    let mut best_score = u16::MAX;
    let mut stagnant_generations: usize = 0;
    let mut population = genetics::generate_initial_population::<BOARD_SIZE>(params);

    loop {
        population = match genetics::run_simulation::<BOARD_SIZE>(params, board, population) {
            Ok(solution) => {
                let _ = tx.send(Snapshot {
                    generation,
                    board: solution,
                    score: 0,
                });

                return Ok(());
            }
            Err(no_solution_found) => {
                if quit.load(Ordering::Relaxed) {
                    return Err(no_solution_found);
                }

                if no_solution_found.best_score < best_score {
                    best_score = no_solution_found.best_score;
                    stagnant_generations = 0;
                } else {
                    stagnant_generations += 1;
                }

                let _ = tx.try_send(Snapshot {
                    generation,
                    board: no_solution_found.best_board,
                    score: no_solution_found.best_score,
                });

                generation += 1;

                if restart > 0 && stagnant_generations >= restart {
                    best_score = u16::MAX;
                    stagnant_generations = 0;

                    genetics::generate_initial_population::<BOARD_SIZE>(params)
                } else {
                    no_solution_found.next_generation
                }
            }
        };
    }
}

/// Draws simulation snapshots at a fixed cadence and handles keyboard input.
///
/// Returns `Ok(())` when the user quits, or when the simulation thread has
/// exited without a solution (its result is surfaced via the join in `run`).
/// After the solved board is drawn, keeps it displayed until the user quits.
fn render_loop(
    terminal: &mut DefaultTerminal,
    start: Instant,
    rx: &Receiver<Snapshot>,
) -> Result<()> {
    // The last snapshot received, persisted across frames so the display
    // (including the elapsed duration) keeps updating even when no new
    // snapshot arrived this frame.
    let mut latest: Option<Snapshot> = None;

    loop {
        if should_quit(POLL_DURATION_RUNNING)? {
            return Ok(());
        }

        let mut disconnected = false;

        loop {
            match rx.try_recv() {
                Ok(snapshot) => latest = Some(snapshot),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }

        if let Some(ref snapshot) = latest {
            terminal.draw(|frame: &mut Frame| {
                frame.render_widget(widget(start, snapshot), frame.area());
            })?;

            if snapshot.score == 0 {
                loop {
                    if should_quit(POLL_DURATION_DONE)? {
                        return Ok(());
                    }
                }
            }
        }

        if disconnected {
            return Ok(());
        }
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
fn widget(start: Instant, snapshot: &Snapshot) -> Text<'static> {
    Text::raw(format!(
        "Duration: {:?}\nGeneration: {}\nScore: {}\n\n{}",
        start.elapsed(),
        snapshot.generation,
        snapshot.score,
        snapshot.board,
    ))
}
