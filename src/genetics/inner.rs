#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use crate::sudoku::{Board, Row};
use arrayvec::ArrayVec;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng, distr::Uniform};
use rand_pcg::Pcg64Mcg;
use rayon::iter::Zip;
use rayon::prelude::*;
use rayon::vec::IntoIter;

pub fn next_generation<const N: usize>(
    params: &super::GAParams,
    population_scores: Vec<(Board<N>, u8)>,
) -> Vec<Board<N>> {
    make_parents(natural_selection(params, population_scores))
        .flat_map(|parents| make_children::<N>(params, parents))
        .collect()
}

fn natural_selection<const N: usize>(
    params: &super::GAParams,
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
