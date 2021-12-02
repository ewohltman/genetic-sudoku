#![warn(
clippy::all,
// clippy::restriction,
clippy::pedantic,
clippy::nursery,
clippy::cargo,
)]

use super::errors::InvalidSolution;
use super::sudoku::{Board, Row};
use parking_lot::Mutex;
use rand::rngs::ThreadRng;
use rand::{distributions::Uniform, thread_rng, Rng};
use rayon::iter::Zip;
use rayon::prelude::*;
use rayon::vec::IntoIter;

const MUTATION_RATE: u8 = 5; // Percent

#[must_use]
pub fn seed_initial_candidates() -> Vec<Board> {
    let mut rng = thread_rng();
    let mut candidates = Vec::new();

    for _ in 0..100 {
        candidates.push(generate_candidate(&mut rng, Uniform::from(1..10)));
    }

    candidates
}

pub fn generate_candidate(rng: &mut ThreadRng, uniform_range: Uniform<u8>) -> Board {
    let mut solution = Board(Vec::new());

    for _ in 0..9 {
        let mut row = Row(Vec::new());

        for _ in 0..9 {
            row.0.push(rng.sample(uniform_range));
        }

        solution.0.push(row);
    }

    solution
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
pub fn run_simulation(base: &Board, candidates: Vec<Board>) -> Result<Vec<Board>, InvalidSolution> {
    let fitness_scores: Mutex<Vec<(Board, u8)>> = Mutex::new(Vec::with_capacity(candidates.len()));

    let valid_solutions: Vec<Board> = candidates
        .into_par_iter()
        .map(|candidate| -> Result<Option<Board>, InvalidSolution> {
            let solution = base.overlay(&candidate)?;
            let score = solution.fitness();

            if score == 0 {
                return Ok(Some(solution));
            }

            fitness_scores.lock().push((solution, score));

            Ok(None)
        })
        .collect::<Result<Vec<Option<Board>>, InvalidSolution>>()?
        .into_iter()
        .flatten()
        .collect();

    if !valid_solutions.is_empty() {
        if let Some(valid_solution) = valid_solutions.first() {
            return Ok(vec![valid_solution.clone()]);
        }
    }

    let fitness_scores = fitness_scores.lock().to_vec();

    Ok(next_candidates(fitness_scores))
}

fn next_candidates(fitness_scores: Vec<(Board, u8)>) -> Vec<Board> {
    let survivors: Vec<Board> = apply_natural_selection(fitness_scores);
    let parents: Zip<IntoIter<Board>, IntoIter<Board>> = make_parents(survivors);

    parents.map(make_children).flatten().collect()
}

fn apply_natural_selection(mut fitness_scores: Vec<(Board, u8)>) -> Vec<Board> {
    fitness_scores.par_sort_unstable_by_key(|(_, score)| *score);

    fitness_scores
        .drain(..fitness_scores.len() / 2)
        .collect::<Vec<(Board, u8)>>()
        .into_par_iter()
        .map(|(survivor, _)| survivor)
        .collect()
}

fn make_parents(survivors: Vec<Board>) -> Zip<IntoIter<Board>, IntoIter<Board>> {
    let parents: Vec<(Option<Board>, Option<Board>)> = survivors
        .into_par_iter()
        .enumerate()
        .map(|(i, survivor)| -> (Option<Board>, Option<Board>) {
            match i % 2 {
                0 => (Some(survivor), None),
                1 => (None, Some(survivor)),
                _ => (None, None),
            }
        })
        .collect();

    let half_parents = parents.len() / 2;
    let mut parents_x: Vec<Board> = Vec::with_capacity(half_parents);
    let mut parents_y: Vec<Board> = Vec::with_capacity(half_parents);

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

fn make_children(parents: (Board, Board)) -> Vec<Board> {
    let Board(parent_x) = parents.0;
    let Board(parent_y) = parents.1;
    let inherits_from: Uniform<u8> = Uniform::from(0..2);
    let mutation_range: Uniform<u8> = Uniform::from(0..101);
    let mutation_values: Uniform<u8> = Uniform::from(1..10);
    let mut rng = thread_rng();
    let mut children = Vec::new();

    for _ in 0..4 {
        let mut child = Vec::new();

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

            child.push(Row(child_values));
        }

        children.push(Board(child));
    }

    children
}
