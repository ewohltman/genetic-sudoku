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

/// Generates an initial population.
///
/// Generates a randomly initialized population.
///
/// # Panics
///
/// Potentially panics if intermediate vectors cannot be converted to fixed
/// sized arrays.
#[must_use]
pub fn generate_initial_population<const N: usize>() -> Vec<Board<N>> {
    let range = Uniform::from(1..=(N as u8));
    let mut rng = thread_rng();
    let mut boards: Vec<Board<N>> = Vec::new();

    for _ in 0..POPULATION_SIZE {
        let mut board: ArrayVec<Row<N>, N> = ArrayVec::new_const();

        for _ in 0..N {
            let mut row: ArrayVec<u8, N> = ArrayVec::new_const();

            for _ in 0..N {
                row.push(rng.sample(range));
            }

            board.push(Row(row.into_inner().unwrap()));
        }

        boards.push(Board(board.into_inner().unwrap()));
    }

    boards
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
    population: Vec<Board<N>>,
) -> Result<Board<N>, NoSolutionFound<N>> {
    let population_scores = Vec::with_capacity(population.len());
    let population_scores = Mutex::<Vec<(Board<N>, u8)>>::new(population_scores);
    let valid_solutions = population
        .into_par_iter()
        .flat_map(|candidate| -> Option<Board<N>> {
            let solution = base.overlay(&candidate);
            let score = solution.fitness();

            if score == 0 {
                return Some(solution);
            }

            population_scores.lock().push((solution, score));

            None
        })
        .collect::<Vec<Board<N>>>();

    if let Some(valid_solution) = valid_solutions.first() {
        return Ok(valid_solution.clone());
    }

    Err(NoSolutionFound {
        next_generation: next_generation(population_scores.into_inner()),
    })
}

fn next_generation<const N: usize>(population_scores: Vec<(Board<N>, u8)>) -> Vec<Board<N>> {
    let survivors = natural_selection(population_scores);
    let children_per_parents = POPULATION_SIZE / ((survivors.len() / 2) as u8);

    make_parents(survivors)
        .into_par_iter()
        .flat_map(|parents| make_children(parents, children_per_parents))
        .collect()
}

fn natural_selection<const N: usize>(mut population_scores: Vec<(Board<N>, u8)>) -> Vec<Board<N>> {
    population_scores.par_sort_unstable_by_key(|(_, score)| *score);

    population_scores
        .drain(..population_scores.len() / 2)
        .collect::<Vec<(Board<N>, u8)>>()
        .into_par_iter()
        .map(|(survivor, _)| survivor)
        .collect()
}

fn make_parents<const N: usize>(
    survivors: Vec<Board<N>>,
) -> Zip<IntoIter<Board<N>>, IntoIter<Board<N>>> {
    let parents: (Vec<Option<Board<N>>>, Vec<Option<Board<N>>>) = survivors
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

    let parents_x: Vec<Board<N>> = parents.0.into_iter().flatten().collect();
    let parents_y: Vec<Board<N>> = parents.1.into_iter().flatten().collect();

    parents_x.into_par_iter().zip(parents_y.into_par_iter())
}

fn make_children<const N: usize>(
    parents: (Board<N>, Board<N>),
    children_per_parents: u8,
) -> Vec<Board<N>> {
    let Board(parent_x) = parents.0;
    let Board(parent_y) = parents.1;
    drop(parents);

    let inherits_from_range: Uniform<u8> = Uniform::from(0..=1);
    let mutation_chance_range: Uniform<u8> = Uniform::from(0..=100);
    let mutation_values_range: Uniform<u8> = Uniform::from(1..=(N as u8));
    let mut rng = thread_rng();
    let mut children: Vec<Board<N>> = Vec::new();

    for _ in 0..children_per_parents {
        let mut child: ArrayVec<Row<N>, N> = ArrayVec::new_const();

        for i in 0..N {
            let Row(x_values) = parent_x[i];
            let Row(y_values) = parent_y[i];
            let mut child_values: ArrayVec<u8, N> = ArrayVec::new_const();

            for j in 0..x_values.len() {
                let mutation_chance = rng.sample(mutation_chance_range);

                if mutation_chance <= MUTATION_RATE {
                    child_values.push(rng.sample(mutation_values_range));
                } else {
                    match rng.sample(inherits_from_range) {
                        0 => child_values.push(x_values[j]),
                        1 => child_values.push(y_values[j]),
                        _ => (),
                    }
                }
            }

            child.push(Row(child_values.into_inner().unwrap()));
        }

        children.push(Board(child.into_inner().unwrap()));
    }

    children
}
