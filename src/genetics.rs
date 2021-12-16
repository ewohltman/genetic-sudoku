#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use super::errors::NoSolutionFound;
use super::sudoku::{Board, Row};
use arrayvec::ArrayVec;
use rand::{distributions::Uniform, thread_rng, Rng};
use rayon::iter::Zip;
use rayon::prelude::*;
use rayon::vec::IntoIter;

pub const POPULATION: usize = 100;
const NUM_SURVIVORS: usize = POPULATION / 2;
const NUM_PARENT_PAIRS: usize = NUM_SURVIVORS / 2;
const NUM_CHILDREN_PER_PARENT_PAIRS: usize = POPULATION / NUM_PARENT_PAIRS;
const MUTATION_RATE: u8 = 5; // Percent

/// Generates an initial population.
///
/// Generates a randomly initialized population.
///
/// # Panics
///
/// Potentially panics if intermediate vectors cannot be converted to fixed
/// sized arrays.
#[inline]
#[must_use]
pub fn generate_initial_population<const N: usize, const M: usize>() -> ArrayVec<Board<N>, M> {
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let range = Uniform::from(1..=max_digit);
    let mut rng = thread_rng();
    let mut boards: ArrayVec<Board<N>, M> = ArrayVec::new();

    for _ in 0..POPULATION {
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
/// Evaluates the given `population` fitness against the `base` Board to find
/// the closest to correct solutions. Returns a valid solution.
///
/// # Arguments
///
/// * `base` - The base Board to find solutions for
/// * `population` - The population to evaluate fitness for
///
/// # Errors
///
/// Will return `Err(NoSolutionFound)` containing the next generation if a
/// valid solution was not found.
#[inline]
pub fn run_simulation<const N: usize, const M: usize>(
    base: &Board<N>,
    population: ArrayVec<Board<N>, M>,
) -> Result<Board<N>, NoSolutionFound<N, M>> {
    let population_scores: Result<Vec<(Board<N>, u8)>, Board<N>> = population
        .into_par_iter()
        .map(|candidate| -> Result<(Board<N>, u8), Board<N>> {
            let solution = base.overlay(candidate);
            let score = solution.fitness();
            if score == 0 {
                Err(solution)
            } else {
                Ok((solution, score))
            }
        })
        .collect();
    drop(population);

    match population_scores {
        Err(valid_solution) => Ok(valid_solution),
        Ok(population) => {
            let candidates: ArrayVec<(Board<N>, u8), M> = population.into_iter().collect();
            let next_generation = next_generation(candidates);
            Err(NoSolutionFound { next_generation })
        }
    }
}

fn next_generation<const N: usize, const M: usize>(
    population_scores: ArrayVec<(Board<N>, u8), M>,
) -> ArrayVec<Board<N>, M> {
    ArrayVec::from_iter(
        make_parents(natural_selection(population_scores))
            .into_par_iter()
            .flat_map(make_children)
            .collect::<Vec<Board<N>>>(),
    )
}

fn natural_selection<const N: usize, const M: usize>(
    mut population_scores: ArrayVec<(Board<N>, u8), M>,
) -> ArrayVec<Board<N>, NUM_SURVIVORS> {
    population_scores.par_sort_unstable_by_key(|(_, score)| *score);

    ArrayVec::from_iter(
        population_scores
            .drain(..NUM_SURVIVORS)
            .collect::<Vec<(Board<N>, u8)>>()
            .into_par_iter()
            .map(|(survivor, _)| survivor)
            .collect::<Vec<Board<N>>>(),
    )
}

fn make_parents<const N: usize, const M: usize>(
    survivors: ArrayVec<Board<N>, M>,
) -> Zip<IntoIter<Board<N>>, IntoIter<Board<N>>> {
    let parents: (Vec<Option<Board<N>>>, Vec<Option<Board<N>>>) = survivors
        .into_par_iter()
        .enumerate()
        .map(|(i, survivor)| -> (Option<Board<N>>, Option<Board<N>>) {
            match i % 2 {
                0 => (Some(*survivor), None),
                1 => (None, Some(*survivor)),
                _ => (None, None),
            }
        })
        .collect();
    drop(survivors);

    let parents_x: Vec<Board<N>> = parents.0.into_iter().flatten().collect();
    let parents_y: Vec<Board<N>> = parents.1.into_iter().flatten().collect();

    parents_x.into_par_iter().zip(parents_y.into_par_iter())
}

fn make_children<const N: usize>(parents: (Board<N>, Board<N>)) -> Vec<Board<N>> {
    let Board(parent_x) = parents.0;
    let Board(parent_y) = parents.1;

    let inherits_from_range: Uniform<u8> = Uniform::from(0..=1);
    let mutation_chance_range: Uniform<u8> = Uniform::from(0..=100);
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let mutation_values_range: Uniform<u8> = Uniform::from(1..=max_digit);
    let mut rng = thread_rng();
    let mut children: ArrayVec<Board<N>, NUM_CHILDREN_PER_PARENT_PAIRS> = ArrayVec::new_const();

    for _ in 0..NUM_CHILDREN_PER_PARENT_PAIRS {
        let mut child: ArrayVec<Row<N>, N> = ArrayVec::new_const();

        for i in 0..N {
            let Row(x_values) = parent_x[i];
            let Row(y_values) = parent_y[i];
            let mut child_values: ArrayVec<u8, N> = ArrayVec::new_const();

            for j in 0..N {
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

    children.to_vec()
}
