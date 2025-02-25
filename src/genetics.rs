#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use super::errors::NoSolutionFound;
use super::sudoku::{Board, Row};
use arrayvec::ArrayVec;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng, distr::Uniform};
use rand_pcg::Pcg64Mcg;
use rayon::iter::Zip;
use rayon::prelude::*;
use rayon::vec::IntoIter;

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
pub fn generate_initial_population<const N: usize, const M: usize>(
    params: &GAParams,
) -> Vec<Board<N>> {
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let values_range = Uniform::new_inclusive(1, max_digit).expect("invalid value range");
    let mut boards: Vec<Board<N>> = Vec::with_capacity(M);

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
pub fn run_simulation<const N: usize, const M: usize>(
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
            next_generation: next_generation::<N, M>(params, generation, population_scores),
        }),
    }
}

fn next_generation<const N: usize, const M: usize>(
    params: &GAParams,
    generation: usize,
    population_scores: Vec<(Board<N>, u8)>,
) -> Vec<Board<N>> {
    if let Some(restart) = params.restart {
        if (generation % restart == 0) && (generation != 0) {
            return generate_initial_population::<N, M>(params);
        }
    }

    make_parents(natural_selection(params, population_scores))
        .flat_map(|parents| make_children::<N, M>(params, parents))
        .collect()
}

fn natural_selection<const N: usize>(
    params: &GAParams,
    mut population_scores: Vec<(Board<N>, u8)>,
) -> Vec<Board<N>> {
    population_scores.par_sort_by_key(|(_, score)| *score);

    population_scores
        .drain(..params.num_survivors)
        .map(|(survivor, _)| survivor)
        .collect()
}

type ScoreBoard<const N: usize> = Vec<(usize, Board<N>)>;

fn make_parents<const N: usize>(
    survivors: Vec<Board<N>>,
) -> Zip<IntoIter<Board<N>>, IntoIter<Board<N>>> {
    let (parent_x, parent_y): (ScoreBoard<N>, ScoreBoard<N>) = survivors
        .into_iter()
        .enumerate()
        .partition(|(i, _)| i % 2 == 0);

    parent_x
        .into_iter()
        .map(|(_, board)| board)
        .collect::<Vec<Board<N>>>()
        .into_par_iter()
        .zip(
            parent_y
                .into_iter()
                .map(|(_, board)| board)
                .collect::<Vec<Board<N>>>()
                .into_par_iter(),
        )
}

fn make_children<const N: usize, const M: usize>(
    params: &GAParams,
    parents: (Board<N>, Board<N>),
) -> Vec<Board<N>> {
    let Board(parent_x) = parents.0;
    let Board(parent_y) = parents.1;
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let values_range = Uniform::new_inclusive(1, max_digit).expect("invalid value range");
    let mutation_rate = f64::from(params.mutation_rate);

    (0..params.num_children_per_parent_pairs)
        .into_par_iter()
        .map(|_| {
            let mut rng = Pcg64Mcg::from_rng(&mut StdRng::from_os_rng());
            let mut child: ArrayVec<Row<N>, N> = ArrayVec::new_const();

            for i in 0..N {
                let Row(x_values) = parent_x[i];
                let Row(y_values) = parent_y[i];
                let mut child_values: ArrayVec<u8, N> = ArrayVec::new_const();

                for j in 0..N {
                    if rng.random_bool(mutation_rate) {
                        child_values.push(rng.sample(values_range));
                        continue;
                    }

                    if rng.random_bool(0.5) {
                        child_values.push(x_values[j]);
                    } else {
                        child_values.push(y_values[j]);
                    }
                }

                child.push(Row(child_values.into_inner().unwrap()));
            }

            Board(child.into_inner().unwrap())
        })
        .collect()
}
