mod inner;

use crate::errors::NoSolutionFound;
use crate::sudoku::{Board, Row};
use rand::{RngExt, SeedableRng, distr::Uniform};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;
use std::array;

pub const MAX_POPULATION: usize = 100_000;

pub struct GAParams {
    population: usize,
    num_survivors: usize,
    num_children_per_parent_pairs: usize,
    mutation_rate: f32,
}

impl GAParams {
    /// Returns new GA parameters.
    ///
    /// # Arguments
    ///
    /// * `population` - the size of the population to use
    /// * `selection_rate` - the fraction of survivors per generation
    /// * `mutation_rate` - the rate at which values should mutate
    ///
    /// # Panics
    ///
    /// Panics if the given population is greater than `MAX_POPULATION`, if
    /// `selection_rate` is not in `(0.0, 1.0]`, if `mutation_rate` is not in
    /// `[0.0, 1.0]`, or if `population * selection_rate` yields fewer than
    /// two survivors.
    #[inline]
    #[must_use]
    pub fn new(population: usize, selection_rate: f32, mutation_rate: f32) -> Self {
        assert!(
            population <= MAX_POPULATION,
            "population must not exceed {MAX_POPULATION}"
        );
        assert!(
            selection_rate > 0.0 && selection_rate <= 1.0,
            "selection_rate must be in (0.0, 1.0]"
        );
        assert!(
            (0.0..=1.0).contains(&mutation_rate),
            "mutation_rate must be in [0.0, 1.0]"
        );

        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss
        )]
        let num_survivors = (population as f32 * selection_rate).floor() as usize;

        assert!(
            num_survivors >= 2,
            "population * selection_rate must yield at least two survivors"
        );

        let num_parent_pairs = num_survivors / 2;
        let num_children_per_parent_pairs = population / num_parent_pairs;

        Self {
            population,
            num_survivors,
            num_children_per_parent_pairs,
            mutation_rate,
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
/// Panics if the board size, N, exceeds 255.
#[inline]
#[must_use]
pub fn generate_initial_population<const N: usize>(params: &GAParams) -> Vec<Board<N>> {
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let values_range = Uniform::new_inclusive(1, max_digit).expect("invalid value range");
    let mut rng = Pcg64Mcg::from_rng(&mut rand::rng());

    (0..params.population)
        .map(|_| {
            Board(array::from_fn(|_| {
                Row(array::from_fn(|_| rng.sample(values_range)))
            }))
        })
        .collect()
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
/// Will return `Err(NoSolutionFound)` containing the next generation, along
/// with the best candidate board and its score, if a valid solution was not
/// found.
///
/// # Panics
///
/// Panics if the given population is empty.
#[inline]
pub fn run_simulation<const N: usize>(
    params: &GAParams,
    base: &Board<N>,
    population: Vec<Board<N>>,
) -> Result<Board<N>, NoSolutionFound<N>> {
    let population_scores: Vec<(Board<N>, u16)> = population
        .into_par_iter()
        .map(|candidate| -> (Board<N>, u16) {
            let solution = base.overlay(&candidate);
            let score = solution.fitness();

            (solution, score)
        })
        .collect();

    if let Some(solution) = population_scores
        .iter()
        .find(|board_score| board_score.1 == 0)
    {
        return Ok(solution.0);
    }

    let (best_board, best_score) = population_scores
        .iter()
        .copied()
        .min_by_key(|(_, score)| *score)
        .expect("population must not be empty");

    Err(NoSolutionFound {
        best_board,
        best_score,
        next_generation: inner::next_generation::<N>(params, population_scores),
    })
}
