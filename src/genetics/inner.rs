use crate::sudoku::{Board, Row};
use rand::{RngExt, SeedableRng, distr::Uniform};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;
use std::array;

pub fn next_generation<const N: usize>(
    params: &super::GAParams,
    population_scores: Vec<(Board<N>, u16)>,
) -> Vec<Board<N>> {
    let survivors = natural_selection(params, population_scores);

    survivors
        .par_chunks_exact(2)
        .flat_map(|parents| make_children::<N>(params, (parents[0], parents[1])))
        .collect()
}

fn natural_selection<const N: usize>(
    params: &super::GAParams,
    mut population_scores: Vec<(Board<N>, u16)>,
) -> Vec<Board<N>> {
    population_scores.par_sort_by_key(|(_, score)| *score);
    population_scores.truncate(params.num_survivors);

    population_scores
        .into_iter()
        .map(|(survivor, _)| survivor)
        .collect()
}

fn make_children<const N: usize>(
    params: &super::GAParams,
    parents: (Board<N>, Board<N>),
) -> Vec<Board<N>> {
    let Board(parent_x) = parents.0;
    let Board(parent_y) = parents.1;
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let values_range = Uniform::new_inclusive(1, max_digit).expect("invalid value range");
    let mutation_rate = f64::from(params.mutation_rate);

    (0..params.num_children_per_parent_pairs)
        .into_par_iter()
        .map_init(
            || Pcg64Mcg::from_rng(&mut rand::rng()),
            |rng, _| {
                Board(array::from_fn(|i| {
                    let Row(x_values) = parent_x[i];
                    let Row(y_values) = parent_y[i];

                    Row(array::from_fn(|j| {
                        if rng.random_bool(mutation_rate) {
                            rng.sample(values_range)
                        } else if rng.random_bool(0.5) {
                            x_values[j]
                        } else {
                            y_values[j]
                        }
                    }))
                }))
            },
        )
        .collect()
}
