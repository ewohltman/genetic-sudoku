pub mod errors;

use errors::{ErrorType, InvalidSolution};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

const BOARD_SIZE: usize = 9;

#[derive(Clone)]
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

#[derive(Clone)]
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
        let mut duplicates: u8 = 0;

        for row in self.0.iter() {
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

fn apply_overlay<T, F>(base: &[T], overlay: &[T], f: F) -> Result<Vec<T>, InvalidSolution>
where
    F: Fn((&T, &T)) -> Result<T, InvalidSolution>,
{
    match overlay.len() {
        BOARD_SIZE => base.iter().zip(overlay.iter()).map(f).collect(),
        size => Err(InvalidSolution::new(ErrorType::InvalidSize(size))),
    }
}
