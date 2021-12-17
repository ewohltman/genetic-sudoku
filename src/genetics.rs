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

pub const MAX_POPULATION: usize = 100;

pub struct GAParams {
    population: usize,
    num_survivors: usize,
    num_children_per_parent_pairs: usize,
    mutation_rate: f32,
    restart: Option<u64>,
}

impl GAParams {
    /// Returns new GA parameters.
    ///
    /// # Arguments
    ///
    /// * `population` - the size of the population to use
    /// * `frac_reduction` - the fractional value of survivors per generation
    /// * `mutation_rate` - the rate at which values should mutate
    /// * `restart` - the number of generations before a population restart
    ///
    /// # Panics
    /// Panics if the given population is greater than `MAX_POPULATION`.
    #[inline]
    #[must_use]
    pub fn new(
        population: usize,
        frac_reduction: f32,
        mutation_rate: f32,
        restart: Option<u64>,
    ) -> Self {
        assert!(population <= MAX_POPULATION);
        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss
        )]
        let num_survivors = (population as f32 * frac_reduction).floor() as usize;
        let num_parent_pairs = num_survivors / 2;
        let num_children_per_parent_pairs = population / num_parent_pairs;
        Self {
            population,
            num_survivors,
            num_children_per_parent_pairs,
            mutation_rate,
            restart,
        }
    }
}

/// Generates an initial population.
///
/// Generates a randomly initialized population.
///
/// # Arguments
///
/// * `params` - GA parameters
///
/// # Panics
///
/// Potentially panics if intermediate vectors cannot be converted to fixed
/// sized arrays.
#[inline]
#[must_use]
pub fn generate_initial_population<const N: usize, const M: usize>(
    params: &GAParams,
) -> ArrayVec<Board<N>, M> {
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let range = Uniform::from(1..=max_digit);
    let mut rng = thread_rng();
    let mut boards: ArrayVec<Board<N>, M> = ArrayVec::new_const();

    for _ in 0..params.population {
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
/// * `params` - GA parameters
/// * `generation` - the current generation counter
/// * `base` - The base Board to find solutions for
/// * `population` - The population to evaluate fitness for
///
/// # Errors
///
/// Will return `Err(NoSolutionFound)` containing the next generation if a
/// valid solution was not found.
#[inline]
pub fn run_simulation<const N: usize, const M: usize>(
    params: &GAParams,
    generation: u64,
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
            let next_generation = next_generation(params, generation, candidates);
            Err(NoSolutionFound { next_generation })
        }
    }
}

fn next_generation<const N: usize, const M: usize>(
    params: &GAParams,
    generation: u64,
    population_scores: ArrayVec<(Board<N>, u8), M>,
) -> ArrayVec<Board<N>, M> {
    if let Some(restart) = params.restart {
        if (generation % restart == 0) && (generation != 0) {
            return generate_initial_population(params);
        }
    }

    ArrayVec::from_iter(
        make_parents(natural_selection(params, population_scores))
            .into_par_iter()
            .flat_map(|pop| make_children::<N, M>(params, pop))
            .collect::<Vec<Board<N>>>(),
    )
}

fn natural_selection<const N: usize, const M: usize>(
    params: &GAParams,
    mut population_scores: ArrayVec<(Board<N>, u8), M>,
) -> ArrayVec<Board<N>, M> {
    population_scores.par_sort_unstable_by_key(|(_, score)| *score);

    ArrayVec::from_iter(
        population_scores
            .drain(..params.num_survivors)
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

fn make_children<const N: usize, const M: usize>(
    params: &GAParams,
    parents: (Board<N>, Board<N>),
) -> Vec<Board<N>> {
    let Board(parent_x) = parents.0;
    let Board(parent_y) = parents.1;

    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let inherits_from_range: Uniform<u8> = Uniform::from(0..=1);
    let mutation_values_range: Uniform<u8> = Uniform::from(1..=max_digit);
    let mut rng = thread_rng();
    let mut children: ArrayVec<Board<N>, M> = ArrayVec::new_const();

    for _ in 0..params.num_children_per_parent_pairs {
        let mut child: ArrayVec<Row<N>, N> = ArrayVec::new_const();

        for i in 0..N {
            let Row(x_values) = parent_x[i];
            let Row(y_values) = parent_y[i];
            let mut child_values: ArrayVec<u8, N> = ArrayVec::new_const();

            for j in 0..N {
                let mutation_chance: f32 = rng.gen();

                if mutation_chance < params.mutation_rate {
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
