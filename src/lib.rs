pub mod errors;

use errors::{ErrorType, InvalidSolution};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

const BOARD_SIZE: usize = 9;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Row(pub Vec<u8>);

impl Display for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Board(pub Vec<Row>);

impl Board {
    pub fn new(rows: Vec<Row>) -> Board {
        Board(rows)
    }

    pub fn overlay(&self, overlay: &Board) -> Result<Board, InvalidSolution> {
        let Board(base) = &self;
        let Board(overlay) = overlay;

        apply_overlay(base, overlay, |(Row(base), Row(overlay))| {
            apply_overlay(base, overlay, |(base, overlay)| match *base {
                0 => Ok(*overlay),
                _ => Ok(*base),
            })
            .map(Row)
        })
        .map(Board)
    }

    pub fn fitness(&self) -> u8 {
        self.count_duplicates() + self.transpose().count_duplicates()
    }

    fn count_duplicates(&self) -> u8 {
        let mut total_duplicates: u8 = 0;

        let duplicates_per_row: Vec<u8> = self
            .0
            .par_iter()
            .map(|row| -> u8 {
                let mut duplicates: u8 = 0;
                let mut hash_map: HashMap<u8, bool> = HashMap::new();

                for value in row.0.iter() {
                    if hash_map.insert(*value, true).is_some() {
                        duplicates += 1;
                    }
                }

                duplicates
            })
            .collect();

        for row_duplicates in duplicates_per_row.into_iter() {
            total_duplicates += row_duplicates;
        }

        total_duplicates
    }

    fn transpose(&self) -> Board {
        let rows = &self.0;
        let row_len = rows[0].0.len();
        let mut transposed: Vec<Row> = Vec::with_capacity(row_len);

        // Initialize the transposed rows
        for _ in 0..row_len {
            transposed.push(Row(Vec::<u8>::with_capacity(rows.len())));
        }

        for row in rows.iter() {
            let row_values = row.0.iter();

            // For each row value index, push row value to transposed row index
            for (j, value) in row_values.enumerate() {
                transposed[j].0.push(*value);
            }
        }

        Board(transposed)
    }
}

impl Default for Board {
    fn default() -> Self {
        Board(vec![
            Row(vec![0, 0, 4, 0, 5, 0, 0, 0, 0]),
            Row(vec![9, 0, 0, 7, 3, 4, 6, 0, 0]),
            Row(vec![0, 0, 3, 0, 2, 1, 0, 4, 9]),
            Row(vec![0, 3, 5, 0, 9, 0, 4, 8, 0]),
            Row(vec![0, 9, 0, 0, 0, 0, 0, 3, 0]),
            Row(vec![0, 7, 6, 0, 1, 0, 9, 2, 0]),
            Row(vec![3, 1, 0, 9, 7, 0, 2, 0, 0]),
            Row(vec![0, 0, 9, 1, 8, 2, 0, 0, 3]),
            Row(vec![0, 0, 0, 0, 6, 0, 1, 0, 0]),
        ])
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in &self.0 {
            writeln!(f, "{}", row)?
        }

        Ok(())
    }
}

pub fn al_escargot() -> Vec<Row> {
    vec![
        Row(vec![1, 0, 0, 0, 0, 7, 0, 9, 0]),
        Row(vec![0, 3, 0, 0, 2, 0, 0, 0, 8]),
        Row(vec![0, 0, 9, 6, 0, 0, 5, 0, 0]),
        Row(vec![0, 0, 5, 3, 0, 0, 9, 0, 0]),
        Row(vec![0, 1, 0, 0, 8, 0, 0, 0, 2]),
        Row(vec![6, 0, 0, 0, 0, 4, 0, 0, 0]),
        Row(vec![3, 0, 0, 0, 0, 0, 0, 1, 0]),
        Row(vec![0, 4, 0, 0, 0, 0, 0, 0, 7]),
        Row(vec![0, 0, 7, 0, 0, 0, 3, 0, 0]),
    ]
}

pub fn al_escargot_2() -> Vec<Row> {
    vec![
        Row(vec![0, 0, 5, 3, 0, 0, 0, 0, 0]),
        Row(vec![8, 0, 0, 0, 0, 0, 0, 2, 0]),
        Row(vec![0, 7, 0, 0, 1, 0, 5, 0, 0]),
        Row(vec![4, 0, 0, 0, 0, 5, 3, 0, 0]),
        Row(vec![0, 1, 0, 0, 7, 0, 0, 0, 6]),
        Row(vec![0, 0, 3, 2, 0, 0, 0, 8, 0]),
        Row(vec![0, 6, 0, 5, 0, 0, 0, 0, 9]),
        Row(vec![0, 0, 4, 0, 0, 0, 0, 3, 0]),
        Row(vec![0, 0, 0, 0, 0, 9, 7, 0, 0]),
    ]
}

fn apply_overlay<T, F>(base: &[T], overlay: &[T], f: F) -> Result<Vec<T>, InvalidSolution>
where
    F: Fn((&T, &T)) -> Result<T, InvalidSolution>,
{
    match overlay.len() {
        BOARD_SIZE => base.iter().zip(overlay.iter()).map(f).collect(),
        size => Err(InvalidSolution::new(ErrorType::InvalidSize(size))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_fitness() {
        let good_board = good_test_board();
        let bad_board = bad_test_board();

        assert_eq!(0, good_board.fitness());
        assert_eq!(8, bad_board.fitness());
    }

    #[test]
    fn test_board_transpose() {
        let board: Board = good_test_board();
        let expected: Board = Board(vec![
            Row(vec![1, 2, 3]),
            Row(vec![2, 3, 4]),
            Row(vec![3, 4, 1]),
            Row(vec![4, 1, 2]),
        ]);

        assert_eq!(expected, board.transpose())
    }

    fn good_test_board() -> Board {
        Board(vec![
            Row(vec![1, 2, 3, 4]),
            Row(vec![2, 3, 4, 1]),
            Row(vec![3, 4, 1, 2]),
        ])
    }

    fn bad_test_board() -> Board {
        Board(vec![
            Row(vec![1, 2, 3, 4]),
            Row(vec![1, 2, 3, 4]),
            Row(vec![1, 2, 3, 4]),
        ])
    }
}
