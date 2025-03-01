mod inner;

use crate::errors::NoSolutionFound;
use crate::sudoku::{Board, Row};
use arrayvec::ArrayVec;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng, distr::Uniform};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;

pub const MAX_POPULATION: usize = 100_000;

pub struct GAParams {
    population: usize,
    num_survivors: usize,
    num_children_per_parent_pairs: usize,
    mutation_rate: f32,
    restart: Option<usize>,
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
        selection_rate: f32,
        mutation_rate: f32,
        restart: Option<usize>,
    ) -> Self {
        assert!(population <= MAX_POPULATION);

        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss
        )]
        let num_survivors = (population as f32 * selection_rate).floor() as usize;
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
pub fn generate_initial_population<const N: usize>(params: &GAParams) -> Vec<Board<N>> {
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let values_range = Uniform::new_inclusive(1, max_digit).expect("invalid value range");
    let mut boards: Vec<Board<N>> = Vec::with_capacity(params.population);

    for _ in 0..params.population {
        let mut board: ArrayVec<Row<N>, N> = ArrayVec::new_const();

        for _ in 0..N {
            let mut row: ArrayVec<u8, N> = ArrayVec::new_const();
            let rng = Pcg64Mcg::from_rng(&mut StdRng::from_os_rng());
            let values: Vec<u8> = rng.sample_iter(values_range).take(N).collect();

            for item in &values {
                row.push(*item);
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
pub fn run_simulation<const N: usize>(
    params: &GAParams,
    generation: usize,
    base: &Board<N>,
    population: Vec<Board<N>>,
) -> Result<Board<N>, NoSolutionFound<N>> {
    let population_scores: Vec<(Board<N>, u8)> = population
        .into_par_iter()
        .map(|candidate| -> (Board<N>, u8) {
            let solution = base.overlay(&candidate);
            let score = solution.fitness();

            (solution, score)
        })
        .collect();

    #[allow(clippy::option_if_let_else)] // Prevent cloning population_scores.
    match population_scores
        .iter()
        .find(|board_score| board_score.1 == 0)
    {
        Some(solution) => Ok(solution.0),
        None => Err(NoSolutionFound {
            next_generation: inner::next_generation::<N>(params, generation, population_scores),
        }),
    }
}
