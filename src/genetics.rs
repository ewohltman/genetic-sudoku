#![warn(
clippy::all,
// clippy::restriction,
clippy::pedantic,
clippy::nursery,
clippy::cargo,
)]

use super::errors::NoSolutionFound;
use super::sudoku::{Board, Row};
use arrayvec::ArrayVec;
use parking_lot::Mutex;
use rand::{distributions::Uniform, thread_rng, Rng};
use rayon::iter::Zip;
use rayon::prelude::*;
use rayon::vec::IntoIter;

const MUTATION_RATE: u8 = 5; // Percent
const POPULATION_SIZE: u8 = 100;

#[must_use]
pub fn generate_initial_boards<const N: usize>() -> Vec<Board<N>> {
    let mut boards: Vec<Board<N>> = Vec::new();

    for _ in 0..POPULATION_SIZE {
        boards.push(generate_initial_board());
    }

    boards
}

/// Generate an initial board.
///
/// Generates a randomly initialized board.
///
/// # Panics
///
/// Potentially panics if intermediate vectors cannot be converted to fixed
/// sized arrays.
#[must_use]
pub fn generate_initial_board<const N: usize>() -> Board<N> {
    let range = Uniform::from(1..=(N as u8));
    let mut rng = thread_rng();
    let mut board: ArrayVec<Row<N>, N> = ArrayVec::new_const();

    for _ in 0..N {
        let mut row: ArrayVec<u8, N> = ArrayVec::new_const();

        for _ in 0..N {
            row.push(rng.sample(range));
        }

        board.push(Row(row.into_inner().unwrap()));
    }

    Board(board.into_inner().unwrap())
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
pub fn run_simulation<const N: usize>(
    base: &Board<N>,
    candidates: Vec<Board<N>>,
) -> Result<Board<N>, NoSolutionFound<N>> {
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
            return Ok(valid_solution.clone());
        }
    }

    let fitness_scores: Vec<(Board<N>, u8)> = fitness_scores.lock().to_vec();

    Err(NoSolutionFound {
        next: next_candidates(fitness_scores),
    })
}

fn next_candidates<const N: usize>(fitness_scores: Vec<(Board<N>, u8)>) -> Vec<Board<N>> {
    let survivors: Vec<Board<N>> = apply_natural_selection(fitness_scores);
    let children_per_parents = POPULATION_SIZE / ((survivors.len() / 2) as u8);
    let parents: Zip<IntoIter<Board<N>>, IntoIter<Board<N>>> = make_parents(survivors);

    parents
        .map(|parents| make_children(&parents, children_per_parents))
        .flatten()
        .collect()
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

fn make_children<const N: usize>(
    parents: &(Board<N>, Board<N>),
    children_per_parents: u8,
) -> Vec<Board<N>> {
    let Board(parent_x) = parents.0;
    let Board(parent_y) = parents.1;
    let inherits_from: Uniform<u8> = Uniform::from(0..=1);
    let mutation_values: Uniform<u8> = Uniform::from(1..=(N as u8));
    let mutation_chance: Uniform<u8> = Uniform::from(0..=100);
    let mut rng = thread_rng();
    let mut children: Vec<Board<N>> = Vec::new();

    for _ in 0..children_per_parents {
        let mut child: Vec<Row<N>> = Vec::new();

        for i in 0..parent_x.len() {
            let Row(x_values) = parent_x[i];
            let Row(y_values) = parent_y[i];
            let mut child_values: Vec<u8> = Vec::new();

            for j in 0..x_values.len() {
                let mutation_chance = rng.sample(mutation_chance);

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
