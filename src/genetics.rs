#![warn(
clippy::all,
// clippy::restriction,
clippy::pedantic,
clippy::nursery,
clippy::cargo,
)]

use super::sudoku::{Board, Row};
use parking_lot::Mutex;
use rand::rngs::ThreadRng;
use rand::{distributions::Uniform, thread_rng, Rng};
use rayon::iter::Zip;
use rayon::prelude::*;
use rayon::vec::IntoIter;

const MUTATION_RATE: u8 = 5; // Percent

#[must_use]
pub fn generate_initial_boards<const N: usize>() -> Vec<Board<N>> {
    let mut rng = thread_rng();
    let mut boards: Vec<Board<N>> = Vec::new();

    for _ in 0..100 {
        boards.push(generate_initial_board(&mut rng, Uniform::from(1..10)));
    }

    boards
}

/// Generate an initial board.
///
/// Generates a randomly initialized board..
///
/// # Arguments
///
/// * `rng` - A random number generator
/// * `uniform_range` - A uniform range distribution to randomly select numbers
///   from
///
/// # Panics
///
/// Potentially panics if intermediate vectors cannot be converted to fixed
/// sized arrays.
pub fn generate_initial_board<const N: usize>(
    rng: &mut ThreadRng,
    uniform_range: Uniform<u8>,
) -> Board<N> {
    let mut board: Vec<Row<N>> = Vec::with_capacity(N);

    for _ in 0..N {
        let mut row: Vec<u8> = Vec::with_capacity(N);

        for _ in 0..9 {
            row.push(rng.sample(uniform_range));
        }

        board.push(Row(row.try_into().unwrap()));
    }

    Board(board.try_into().unwrap())
}

/// Runs the simulation.
///
/// Evaluates all the given `candidates` fitness against the `base` Board to
/// find the closest to correct solutions. Returns a Result containing a Vector
/// either with a single element, representing a valid correct solution, or the
/// next generation's candidates to be evaluated.
///
/// # Arguments
///
/// * `base` - The base Board to find solutions for
/// * `candidates` - A Vector of solution candidates
///
/// # Errors
///
/// Will return `Err(InvalidSolution)` if any of the `candidates` are not the
/// same length as `self`.
#[must_use]
pub fn run_simulation<const N: usize>(base: &Board<N>, candidates: Vec<Board<N>>) -> Vec<Board<N>> {
    let fitness_scores: Mutex<Vec<(Board<N>, u8)>> =
        Mutex::new(Vec::with_capacity(candidates.len()));

    let valid_solutions: Vec<Board<N>> = candidates
        .into_par_iter()
        .map(|candidate| -> Option<Board<N>> {
            let solution = base.overlay(&candidate);
            let score = solution.fitness();

            if score == 0 {
                return Some(solution);
            }

            fitness_scores.lock().push((solution, score));

            None
        })
        .flatten()
        .collect::<Vec<Board<N>>>();

    if !valid_solutions.is_empty() {
        if let Some(valid_solution) = valid_solutions.first() {
            return vec![valid_solution.clone()];
        }
    }

    let fitness_scores = fitness_scores.lock().to_vec();

    next_candidates(fitness_scores)
}

fn next_candidates<const N: usize>(fitness_scores: Vec<(Board<N>, u8)>) -> Vec<Board<N>> {
    let survivors: Vec<Board<N>> = apply_natural_selection(fitness_scores);
    let parents: Zip<IntoIter<Board<N>>, IntoIter<Board<N>>> = make_parents(survivors);

    parents.map(make_children).flatten().collect()
}

fn apply_natural_selection<const N: usize>(
    mut fitness_scores: Vec<(Board<N>, u8)>,
) -> Vec<Board<N>> {
    fitness_scores.par_sort_unstable_by_key(|(_, score)| *score);

    fitness_scores
        .drain(..fitness_scores.len() / 2)
        .collect::<Vec<(Board<N>, u8)>>()
        .into_par_iter()
        .map(|(survivor, _)| survivor)
        .collect()
}

fn make_parents<const N: usize>(
    survivors: Vec<Board<N>>,
) -> Zip<IntoIter<Board<N>>, IntoIter<Board<N>>> {
    let parents: Vec<(Option<Board<N>>, Option<Board<N>>)> = survivors
        .into_par_iter()
        .enumerate()
        .map(|(i, survivor)| -> (Option<Board<N>>, Option<Board<N>>) {
            match i % 2 {
                0 => (Some(survivor), None),
                1 => (None, Some(survivor)),
                _ => (None, None),
            }
        })
        .collect();

    let half_parents = parents.len() / 2;
    let mut parents_x: Vec<Board<N>> = Vec::with_capacity(half_parents);
    let mut parents_y: Vec<Board<N>> = Vec::with_capacity(half_parents);

    for (parent_x, parent_y) in parents {
        if let Some(parent) = parent_x {
            parents_x.push(parent);
        }

        if let Some(parent) = parent_y {
            parents_y.push(parent);
        }
    }

    parents_x.into_par_iter().zip(parents_y.into_par_iter())
}

fn make_children<const N: usize>(parents: (Board<N>, Board<N>)) -> Vec<Board<N>> {
    let Board(parent_x) = parents.0;
    let Board(parent_y) = parents.1;
    let inherits_from: Uniform<u8> = Uniform::from(0..2);
    let mutation_range: Uniform<u8> = Uniform::from(0..101);
    let mutation_values: Uniform<u8> = Uniform::from(1..10);
    let mut rng = thread_rng();
    let mut children: Vec<Board<N>> = Vec::new();

    for _ in 0..4 {
        let mut child: Vec<Row<N>> = Vec::new();

        for i in 0..parent_x.len() {
            let Row(x_values) = parent_x[i].clone();
            let Row(y_values) = parent_y[i].clone();
            let mut child_values = Vec::new();

            for j in 0..x_values.len() {
                let mutation_chance = rng.sample(mutation_range);

                if mutation_chance <= MUTATION_RATE {
                    child_values.push(rng.sample(mutation_values));
                } else {
                    match rng.sample(inherits_from) {
                        0 => child_values.push(x_values[j]),
                        1 => child_values.push(y_values[j]),
                        _ => (),
                    }
                }
            }

            child.push(Row(child_values.try_into().unwrap()));
        }

        children.push(Board(child.try_into().unwrap()));
    }

    children
}
