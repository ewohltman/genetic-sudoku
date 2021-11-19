use genetic_sudoku::{errors::InvalidSolution, Board, Row};
use rand::rngs::ThreadRng;
use rand::{distributions::Uniform, thread_rng, Rng};

const TOTAL_GENERATIONS: u16 = 10000;

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
    uniform_range: Uniform<u8>,
    generation: u16,
    candidates: Vec<Board>,
) -> Result<Option<Board>, InvalidSolution> {
    if generation == TOTAL_GENERATIONS {
        return Ok(None);
    }

    println!("Generation: {}", generation);

    let mut fitness_scores: Vec<(Board, u8)> = Vec::new();

    for candidate in candidates.iter() {
        let solution = base.overlay(candidate)?;
        let score = solution.fitness();

        if score == 0 {
            return Ok(Some(solution));
        }

        fitness_scores.push((solution, score));
    }

    fitness_scores.sort_unstable_by_key(|key| key.1);

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

    let children: Vec<Board> = parents_x
        .iter()
        .zip(parents_y.iter())
        .map(|(parent_x, parent_y)| {
            let Board(parent_x_rows) = parent_x;
            let Board(parent_y_rows) = parent_y;
            let mut offspring: Vec<Board> = Vec::new();

            for _ in 0..4 {
                let mut child: Vec<Row> = Vec::new();

                for i in 0..parent_x_rows.len() {
                    let Row(parent_x_values) = parent_x_rows[i].clone();
                    let Row(parent_y_values) = parent_y_rows[i].clone();
                    let mut child_values: Vec<u8> = Vec::new();

                    for j in 0..parent_x_values.len() {
                        match rng.sample(Uniform::new(0, 2)) {
                            0 => child_values.push(parent_x_values[j]),
                            1 => child_values.push(parent_y_values[j]),
                            _ => (),
                        }
                    }

                    child.push(Row(child_values))
                }

                offspring.push(Board(child));
            }

            offspring
        })
        .flatten()
        .collect();

    run_simulation(base, rng, uniform_range, generation + 1, children)
}

fn main() -> Result<(), InvalidSolution> {
    let mut rng = thread_rng();
    let uniform_range = Uniform::from(1..10);
    let base = Board::default();

    let mut candidates = Vec::new();

    for _ in 0..100 {
        candidates.push(generate_candidate(&mut rng, uniform_range));
    }

    match run_simulation(&base, &mut rng, uniform_range, 0, candidates)? {
        Some(solution) => println!("Solution:\n{}", solution),
        None => println!("No solution found"),
    }

    Ok(())
}
