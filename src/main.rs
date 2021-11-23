use genetic_sudoku::{errors::InvalidSolution, Board, Row};
use rand::rngs::ThreadRng;
use rand::{distributions::Uniform, thread_rng, Rng};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;

const MUTATION_RATE: u8 = 5; // Percent

fn seed_initial_candidates(mut rng: &mut ThreadRng, uniform_range: Uniform<u8>) -> Vec<Board> {
    let mut candidates = Vec::new();

    for _ in 0..100 {
        candidates.push(generate_candidate(&mut rng, uniform_range));
    }

    candidates
}

fn generate_candidate(rng: &mut ThreadRng, uniform_range: Uniform<u8>) -> Board {
    let mut solution = Board(Vec::new());

    for _ in 0..9 {
        let mut row = Row(Vec::new());

        for _ in 0..9 {
            row.0.push(rng.sample(uniform_range))
        }

        solution.0.push(row);
    }

    solution
}

fn run_simulation(
    base: &Board,
    rng: &mut ThreadRng,
    candidates: Vec<Board>,
) -> Result<Vec<Board>, InvalidSolution> {
    let fitness_scores = Arc::new(Mutex::new(Vec::<(Board, u8)>::with_capacity(
        candidates.len(),
    )));
    let mut threads: Vec<JoinHandle<Result<Option<Board>, InvalidSolution>>> =
        Vec::with_capacity(candidates.len());

    for candidate in candidates.into_iter() {
        let thread_base = base.clone();
        let thread_fitness_scores = Arc::clone(&fitness_scores);

        threads.push(thread::spawn(
            move || -> Result<Option<Board>, InvalidSolution> {
                let solution = thread_base.overlay(&candidate)?;
                let score = solution.fitness();

                if score == 0 {
                    return Ok(Some(solution));
                }

                thread_fitness_scores
                    .lock()
                    .unwrap()
                    .push((solution, score));

                Ok(None)
            },
        ));
    }

    for handle in threads.into_iter() {
        if let Some(solution) = handle.join().unwrap()? {
            return Ok(vec![solution]);
        }
    }

    let fitness_scores = fitness_scores.lock().unwrap().to_vec();

    Ok(next_candidates(rng, fitness_scores))
}

fn next_candidates(rng: &mut ThreadRng, mut fitness_scores: Vec<(Board, u8)>) -> Vec<Board> {
    fitness_scores.sort_unstable_by_key(|(_, score)| *score);

    let half = fitness_scores.len() / 2;
    let survivors = fitness_scores.drain(..half).map(|(survivor, _)| survivor);
    let mut parents_x = Vec::new();
    let mut parents_y = Vec::new();

    for (i, survivor) in survivors.into_iter().enumerate() {
        match i % 2 {
            0 => parents_x.push(survivor),
            1 => parents_y.push(survivor),
            _ => (),
        };
    }

    let parents_x = parents_x.iter();
    let parents_y = parents_y.iter();
    let parents = parents_x.zip(parents_y);
    let inherits_from: Uniform<u8> = Uniform::from(0..2);
    let mutation_range: Uniform<u8> = Uniform::from(0..101);
    let mutation_values: Uniform<u8> = Uniform::from(1..10);

    parents
        .map(|(parent_x, parent_y)| -> Vec<Board> {
            let Board(parent_x_rows) = parent_x;
            let Board(parent_y_rows) = parent_y;
            let mut children: Vec<Board> = Vec::new();

            for _ in 0..4 {
                let mut child: Vec<Row> = Vec::new();

                for i in 0..parent_x_rows.len() {
                    let Row(parent_x_values) = parent_x_rows[i].clone();
                    let Row(parent_y_values) = parent_y_rows[i].clone();
                    let mut child_values: Vec<u8> = Vec::new();

                    for j in 0..parent_x_values.len() {
                        let mutation_chance = rng.sample(mutation_range);

                        match mutation_chance <= MUTATION_RATE {
                            true => child_values.push(rng.sample(mutation_values)),
                            false => match rng.sample(inherits_from) {
                                0 => child_values.push(parent_x_values[j]),
                                1 => child_values.push(parent_y_values[j]),
                                _ => (),
                            },
                        }
                    }

                    child.push(Row(child_values))
                }

                children.push(Board(child));
            }

            children
        })
        .flatten()
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let base = Board::default();
    let mut rng = thread_rng();

    loop {
        let now = SystemTime::now();
        let mut generation: u64 = 0;
        let mut candidates = seed_initial_candidates(&mut rng, Uniform::from(1..10));

        loop {
            candidates = run_simulation(&base, &mut rng, candidates)?;

            if candidates.len() == 1 {
                println!(
                    "Solution Found: Generation: {}: Duration: {} seconds",
                    generation,
                    now.elapsed().unwrap().as_secs()
                );
                break;
            }

            generation += 1;
        }
    }
}
