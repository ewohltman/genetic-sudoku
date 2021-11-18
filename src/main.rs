use genetic_sudoku::{errors::InvalidSolution, Board, Row};
use rand::{thread_rng, Rng};

fn run_simulation(
    base: &Board,
    generation: usize,
    candidates: Vec<Board>,
) -> Result<(), InvalidSolution> {
    for (run, candidate) in candidates.iter().enumerate() {
        let fitness_score = base.overlay(candidate)?.fitness();
        println!(
            "Generation: {} -> Run: {} -> Fitness score: {}",
            generation, run, fitness_score
        );
    }

    Ok(())
}

fn main() -> Result<(), InvalidSolution> {
    let base = Board::default();
    let mut rng = thread_rng();

    for generation in 0..100 {
        let mut candidates = Vec::new();

        for _ in 0..100 {
            let mut solution = Board(Vec::new());

            for _ in 0..9 {
                let mut row = Row(Vec::new());

                for _ in 0..9 {
                    row.0.push(rng.gen_range(1, 10))
                }

                solution.0.push(row);
            }

            candidates.push(solution);
        }

        run_simulation(&base, generation, candidates)?;
    }

    Ok(())
}
