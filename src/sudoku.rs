#![warn(
clippy::all,
// clippy::restriction,
clippy::pedantic,
clippy::nursery,
clippy::cargo,
)]

use arrayvec::ArrayVec;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Row<const N: usize>(pub [u8; N]);

impl<const N: usize> Default for Row<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> Display for Row<N> {
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
pub struct Board<const N: usize>(pub [Row<N>; N]);

impl<const N: usize> Board<N> {
    #[must_use]
    pub const fn new(rows: [Row<N>; N]) -> Self {
        Self(rows)
    }

    /// Overlays the given `overlay` on top of `self`.
    ///
    /// Returns a Result containing a new Board with the provided `overlay`
    /// on top of `self`.
    ///
    /// # Arguments
    ///
    /// * `overlay` - A Board to overlay on top of `self`
    ///
    /// # Examples
    ///
    /// ```
    /// use genetic_sudoku::sudoku::{Board, Row};
    ///
    /// let base = Board::new([Row([0, 1]), Row([1, 0])]);
    /// let overlay = Board::new([Row([2, 0]), Row([0, 2])]);
    /// let overlaid = base.overlay(&overlay);
    ///
    /// assert_eq!(Board::new([Row([2, 1]), Row([1, 2])]), overlaid);
    /// ```
    #[must_use]
    pub fn overlay(&self, overlay: &Self) -> Self {
        let base_board: &[Row<N>; N] = &self.0;
        let overlay_board: &[Row<N>; N] = &overlay.0;

        let board: [Row<N>; N] =
            apply_overlay(base_board, overlay_board, |(base_row, overlay_row)| {
                let base_row: &[u8; N] = &base_row.0;
                let overlay_row: &[u8; N] = &overlay_row.0;

                let row: [u8; N] =
                    apply_overlay(base_row, overlay_row, |(base_value, overlay_value)| {
                        match *base_value {
                            0 => *overlay_value,
                            _ => *base_value,
                        }
                    });

                Row(row)
            });

        Self(board)
    }

    #[must_use]
    pub fn fitness(&self) -> u8 {
        self.count_duplicates() + self.transpose().count_duplicates()
    }

    fn count_duplicates(&self) -> u8 {
        let mut total_duplicates: u8 = 0;

        for row in self.0 {
            let mut duplicates: u8 = 0;
            let mut hash_map: HashMap<u8, bool> = HashMap::new();

            for value in &row.0 {
                if hash_map.insert(*value, true).is_some() {
                    duplicates += 1;
                }
            }

            total_duplicates += duplicates;
        }

        total_duplicates
    }

    fn transpose(&self) -> Self {
        let rows = &self.0;
        let mut transposed: [Row<N>; N] = [Row::default(); N];

        for (i, row) in rows.iter().enumerate() {
            for (j, value) in row.0.iter().enumerate() {
                transposed[j].0[i] = *value;
            }
        }

        Self(transposed)
    }
}

impl Default for Board<9> {
    fn default() -> Self {
        Self([
            Row([0, 0, 4, 0, 5, 0, 0, 0, 0]),
            Row([9, 0, 0, 7, 3, 4, 6, 0, 0]),
            Row([0, 0, 3, 0, 2, 1, 0, 4, 9]),
            Row([0, 3, 5, 0, 9, 0, 4, 8, 0]),
            Row([0, 9, 0, 0, 0, 0, 0, 3, 0]),
            Row([0, 7, 6, 0, 1, 0, 9, 2, 0]),
            Row([3, 1, 0, 9, 7, 0, 2, 0, 0]),
            Row([0, 0, 9, 1, 8, 2, 0, 0, 3]),
            Row([0, 0, 0, 0, 6, 0, 1, 0, 0]),
        ])
    }
}

impl<const N: usize> Display for Board<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in &self.0 {
            writeln!(f, "{}", row)?;
        }

        Ok(())
    }
}

#[must_use]
pub const fn x4() -> [Row<4>; 4] {
    [
        Row([3, 0, 4, 0]),
        Row([0, 1, 0, 2]),
        Row([0, 4, 0, 3]),
        Row([2, 0, 1, 0]),
    ]
}

#[must_use]
pub const fn x5() -> [Row<5>; 5] {
    [
        Row([4, 2, 0, 0, 0]),
        Row([3, 0, 0, 0, 0]),
        Row([0, 0, 0, 0, 0]),
        Row([0, 0, 0, 0, 4]),
        Row([0, 0, 0, 2, 1]),
    ]
}

#[must_use]
pub const fn al_escargot() -> [Row<9>; 9] {
    [
        Row([1, 0, 0, 0, 0, 7, 0, 9, 0]),
        Row([0, 3, 0, 0, 2, 0, 0, 0, 8]),
        Row([0, 0, 9, 6, 0, 0, 5, 0, 0]),
        Row([0, 0, 5, 3, 0, 0, 9, 0, 0]),
        Row([0, 1, 0, 0, 8, 0, 0, 0, 2]),
        Row([6, 0, 0, 0, 0, 4, 0, 0, 0]),
        Row([3, 0, 0, 0, 0, 0, 0, 1, 0]),
        Row([0, 4, 0, 0, 0, 0, 0, 0, 7]),
        Row([0, 0, 7, 0, 0, 0, 3, 0, 0]),
    ]
}

#[must_use]
pub const fn al_escargot_2() -> [Row<9>; 9] {
    [
        Row([0, 0, 5, 3, 0, 0, 0, 0, 0]),
        Row([8, 0, 0, 0, 0, 0, 0, 2, 0]),
        Row([0, 7, 0, 0, 1, 0, 5, 0, 0]),
        Row([4, 0, 0, 0, 0, 5, 3, 0, 0]),
        Row([0, 1, 0, 0, 7, 0, 0, 0, 6]),
        Row([0, 0, 3, 2, 0, 0, 0, 8, 0]),
        Row([0, 6, 0, 5, 0, 0, 0, 0, 9]),
        Row([0, 0, 4, 0, 0, 0, 0, 3, 0]),
        Row([0, 0, 0, 0, 0, 9, 7, 0, 0]),
    ]
}

fn apply_overlay<T, F, const N: usize>(base: &[T; N], overlay: &[T; N], f: F) -> [T; N]
where
    T: Debug,
    F: Fn((&T, &T)) -> T,
{
    base.iter()
        .zip(overlay.iter())
        .map(f)
        .collect::<ArrayVec<T, N>>()
        .into_inner()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_fitness() {
        let good_board = good_test_board();
        let bad_board = bad_test_board();

        assert_eq!(0, good_board.fitness());
        assert_eq!(12, bad_board.fitness());
    }

    #[test]
    fn test_board_transpose() {
        let board = good_test_board();
        let expected = Board([
            Row([1, 2, 3, 4]),
            Row([2, 3, 4, 1]),
            Row([3, 4, 1, 2]),
            Row([4, 1, 2, 3]),
        ]);

        assert_eq!(expected, board.transpose());
    }

    const fn good_test_board() -> Board<4> {
        Board([
            Row([1, 2, 3, 4]),
            Row([2, 3, 4, 1]),
            Row([3, 4, 1, 2]),
            Row([4, 1, 2, 3]),
        ])
    }

    const fn bad_test_board() -> Board<4> {
        Board([
            Row([1, 2, 3, 4]),
            Row([1, 2, 3, 4]),
            Row([1, 2, 3, 4]),
            Row([1, 2, 3, 4]),
        ])
    }
}
