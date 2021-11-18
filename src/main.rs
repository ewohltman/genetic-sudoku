use genetic_sudoku::{errors::InvalidSolution, Board, Row, Sudoku};

fn main() -> Result<(), InvalidSolution> {
    let base = Sudoku::default();
    let overlay = Board(vec![
        Row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
        Row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
        Row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
        Row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
        Row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
        Row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
        Row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
        Row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
        Row(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
    ]);
    let solution = base.overlay(&overlay)?;
    let fitness_score = solution.fitness();

    println!("Original board:\n{}", base);
    println!("Solution board:\n{}", overlay);
    println!("Overlay:\n{}", solution);
    println!("Solution fitness score: {}", fitness_score);

    Ok(())
}
