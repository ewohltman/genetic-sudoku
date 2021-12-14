#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use arrayvec::ArrayVec;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Row<const N: usize>(pub [u8; N]);

impl<const N: usize> Default for Row<N> {
    #[inline]
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> Display for Row<N> {
    #[inline]
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

#[derive(Debug, Default)]
struct Scorer {
    seen: u64,
    score: u8,
}

impl Scorer {
    fn check(&mut self, digit: u8) {
        let bit = 1 << digit;
        if self.seen & bit != 0 {
            self.score += 1;
        }
        self.seen |= bit;
    }

    const fn score(self) -> u8 {
        self.score
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Board<const N: usize>(pub [Row<N>; N]);

impl<const N: usize> Board<N> {
    #[inline]
    #[must_use]
    pub const fn new(rows: [Row<N>; N]) -> Self {
        Self(rows)
    }

    #[inline]
    #[must_use]
    pub const fn size(&self) -> usize {
        self.0.len()
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
    #[inline]
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

    #[inline]
    #[must_use]
    pub fn fitness(&self) -> u8 {
        self.count_row_duplicates()
            + self.transpose().count_row_duplicates()
            + self.count_box_duplicates()
    }

    #[inline]
    #[must_use]
    pub fn count_row_duplicates(&self) -> u8 {
        let mut total_duplicates: u8 = 0;

        for row in self.0 {
            let mut scorer = Scorer::default();

            for value in &row.0 {
                scorer.check(*value);
            }
            total_duplicates += scorer.score();
        }

        total_duplicates
    }

    fn count_box_duplicates(&self) -> u8 {
        let mut total_duplicates: u8 = 0;

        // XXX This could be a proper integer square root.
        // Realistically these are the only sizes that
        // matter anyhow, and theres no built-in integer
        // sqrt() in Rust.
        let box_size = match N {
            4 => 2,
            9 => 3,
            16 => 4,
            25 => 5,
            _ => panic!("puzzle size N must be one of (2..5)**2"),
        };

        for row in (0..N).step_by(box_size) {
            for col in (0..N).step_by(box_size) {
                let mut scorer = Scorer::default();

                for r in &self.0[row..row+box_size] {
                    for value in &r.0[col..col+box_size] {
                        scorer.check(*value);
                    }
                }
                total_duplicates += scorer.score();
            }
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

impl<const N: usize> Display for Board<N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in &self.0 {
            writeln!(f, "{}", row)?;
        }

        Ok(())
    }
}

#[inline]
#[must_use]
pub const fn default() -> Board<9> {
    Board([
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

#[inline]
#[must_use]
pub const fn al_escargot() -> Board<9> {
    Board([
        Row([1, 0, 0, 0, 0, 7, 0, 9, 0]),
        Row([0, 3, 0, 0, 2, 0, 0, 0, 8]),
        Row([0, 0, 9, 6, 0, 0, 5, 0, 0]),
        Row([0, 0, 5, 3, 0, 0, 9, 0, 0]),
        Row([0, 1, 0, 0, 8, 0, 0, 0, 2]),
        Row([6, 0, 0, 0, 0, 4, 0, 0, 0]),
        Row([3, 0, 0, 0, 0, 0, 0, 1, 0]),
        Row([0, 4, 0, 0, 0, 0, 0, 0, 7]),
        Row([0, 0, 7, 0, 0, 0, 3, 0, 0]),
    ])
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

    const GOOD_BOARD: Board<4> = Board([
        Row([1, 2, 3, 4]),
        Row([2, 3, 4, 1]),
        Row([3, 4, 1, 2]),
        Row([4, 1, 2, 3]),
    ]);

    const GOOD_BOARD_TRANSPOSED: Board<4> = Board([
        Row([1, 2, 3, 4]),
        Row([2, 3, 4, 1]),
        Row([3, 4, 1, 2]),
        Row([4, 1, 2, 3]),
    ]);

    const BAD_BOARD: Board<4> = Board([
        Row([1, 2, 3, 4]),
        Row([1, 2, 3, 4]),
        Row([1, 2, 3, 4]),
        Row([1, 2, 3, 4]),
    ]);

    #[test]
    fn test_board_fitness() {
        assert_eq!(0, GOOD_BOARD.fitness());
        assert_eq!(12, BAD_BOARD.fitness());
    }

    #[test]
    fn test_board_transpose() {
        assert_eq!(GOOD_BOARD_TRANSPOSED, GOOD_BOARD.transpose());
    }
}
