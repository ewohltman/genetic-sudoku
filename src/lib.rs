pub mod errors;

use errors::{ErrorType, InvalidSolution};
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub struct Row(pub Vec<u8>);

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = &self.0;

        for (i, value) in values.iter().enumerate() {
            match values.len() - i {
                1 => write!(f, "{}", value)?,
                _ => write!(f, "{} ", value)?,
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct Board(pub Vec<Row>);

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.0 {
            writeln!(f, "{}", row)?
        }

        Ok(())
    }
}

pub struct Sudoku {
    pub board: Board,
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)
    }
}

impl Default for Sudoku {
    fn default() -> Self {
        Sudoku {
            board: Board(vec![
                Row(vec![0, 0, 4, 0, 5, 0, 0, 0, 0]),
                Row(vec![9, 0, 0, 7, 3, 4, 6, 0, 0]),
                Row(vec![0, 0, 3, 0, 2, 1, 0, 4, 9]),
                Row(vec![0, 3, 5, 0, 9, 0, 4, 8, 0]),
                Row(vec![0, 9, 0, 0, 0, 0, 0, 3, 0]),
                Row(vec![0, 7, 6, 0, 1, 0, 9, 2, 0]),
                Row(vec![3, 1, 0, 9, 7, 0, 2, 0, 0]),
                Row(vec![0, 0, 9, 1, 8, 2, 0, 0, 3]),
                Row(vec![0, 0, 0, 0, 6, 0, 1, 0, 0]),
            ]),
        }
    }
}

impl Sudoku {
    pub fn new(board: Board) -> Sudoku {
        Sudoku { board }
    }

    pub fn overlay(&self, overlay: &Board) -> Result<Sudoku, InvalidSolution> {
        let Board(base) = &self.board;
        let Board(overlay) = overlay;

        Ok(Sudoku {
            board: Board(apply_overlay(base, overlay, overlay_rows)?),
        })
    }

    pub fn fitness(&self) -> usize {
        let mut duplicates: usize = 0;

        for row in self.board.0.iter() {
            let mut hash_map: HashMap<u8, bool> = HashMap::new();

            for value in row.0.iter() {
                if hash_map.insert(*value, true).is_some() {
                    duplicates += 1;
                }
            }
        }

        duplicates
    }
}

fn overlay_rows(pair: (&Row, &Row)) -> Result<Row, InvalidSolution> {
    let Row(base) = pair.0;
    let Row(overlay) = pair.1;

    Ok(Row(apply_overlay(base, overlay, overlay_values)?))
}

fn overlay_values(pair: (&u8, &u8)) -> Result<u8, InvalidSolution> {
    let base = pair.0;
    let overlay = pair.1;

    match *base {
        0 => Ok(*overlay),
        _ => Ok(*base),
    }
}

fn apply_overlay<T, F>(base: &[T], overlay: &[T], f: F) -> Result<Vec<T>, InvalidSolution>
where
    F: Fn((&T, &T)) -> Result<T, InvalidSolution>,
{
    if overlay.len() != 9 {
        return Err(InvalidSolution::new(ErrorType::InvalidSize(overlay.len())));
    }

    base.iter().zip(overlay.iter()).map(f).collect()
}
