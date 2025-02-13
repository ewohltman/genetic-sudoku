#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use arrayvec::ArrayVec;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

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
                1 => write!(f, "{value}")?,
                _ => write!(f, "{value} ")?,
            }
        }

        Ok(())
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

    /// Counts box duplicates.
    ///
    /// Iterates through the Board's sub-boxes to count duplicated values.
    ///
    /// # Panics
    ///
    /// Panics if the size of the Board, N, is not a perfect square >= 4 or <=
    /// 25.
    #[inline]
    #[must_use]
    pub fn count_box_duplicates(&self) -> u8 {
        let mut total_duplicates: u8 = 0;

        // XXX This could be a proper integer square root.
        // Realistically these are the only sizes that
        // matter anyhow, and there is no built-in integer
        // sqrt() in Rust.
        let box_size = match N {
            4 => 2,
            9 => 3,
            16 => 4,
            25 => 5,
            _ => panic!("puzzle size N must be one of (2..5)^2"),
        };

        for row in (0..N).step_by(box_size) {
            for col in (0..N).step_by(box_size) {
                let mut scorer = Scorer::default();

                for r in &self.0[row..row + box_size] {
                    for value in &r.0[col..col + box_size] {
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

    /// Read a board from a file.
    ///
    /// # Errors
    ///
    /// Fails if file is nonexistent, unreadable, or of the wrong size.
    #[inline]
    pub fn read<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let board = std::fs::read_to_string(path)?;
        let format_error =
            || std::io::Error::new(std::io::ErrorKind::InvalidData, "malformed sudoku board");
        let dim = board.lines().next().ok_or_else(format_error)?.len();
        if dim != N {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "wrong board size",
            ));
        }
        let mut board_array: [Row<N>; N] = [<Row<N>>::default(); N];
        let mut chars = board.chars();
        for r in &mut board_array {
            let mut row = Row::default();
            for c in &mut row.0 {
                let ch = chars.next().ok_or_else(format_error)?;
                #[allow(clippy::cast_possible_truncation)]
                let d = ch.to_digit(N as u32 + 1).ok_or_else(format_error)? as u8;
                *c = d;
            }
            *r = row;
            let ch = chars.next().ok_or_else(format_error)?;
            if ch != '\n' {
                return Err(format_error());
            }
        }
        Ok(Self(board_array))
    }
}

impl<const N: usize> Display for Board<N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, row) in self.0.iter().enumerate() {
            if i < self.0.len() - 1 {
                writeln!(f, "{row}")?;
            } else {
                write!(f, "{row}")?;
            }
        }

        Ok(())
    }
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
        Row([3, 4, 1, 2]),
        Row([4, 3, 2, 1]),
        Row([2, 1, 4, 3]),
    ]);

    const GOOD_BOARD_TRANSPOSED: Board<4> = Board([
        Row([1, 3, 4, 2]),
        Row([2, 4, 3, 1]),
        Row([3, 1, 2, 4]),
        Row([4, 2, 1, 3]),
    ]);

    const BAD_BOARD: Board<4> = Board([
        Row([1, 2, 3, 4]),
        Row([1, 2, 3, 4]),
        Row([1, 2, 3, 4]),
        Row([1, 2, 3, 4]),
    ]);

    #[test]
    fn test_scorer() {
        test_scorer_no_duplicates();
        test_scorer_with_duplicates();
    }

    #[test]
    fn test_board_fitness() {
        assert_eq!(0, GOOD_BOARD.fitness());
        assert_eq!(20, BAD_BOARD.fitness());
    }

    #[test]
    fn test_board_transpose() {
        assert_eq!(GOOD_BOARD_TRANSPOSED, GOOD_BOARD.transpose());
    }

    fn test_scorer_no_duplicates() {
        let mut scorer = Scorer::default();

        // Since Scorer.seen is u64, it supports up to 49 before overflowing
        for i in 1..=49 {
            scorer.check(i);
        }

        assert_eq!(0, scorer.score());
    }

    fn test_scorer_with_duplicates() {
        let mut scorer = Scorer::default();

        scorer.check(1);
        scorer.check(1); // One duplicate
        scorer.check(2);
        scorer.check(2); // Two duplicates
        scorer.check(2); // Three duplicates

        assert_eq!(3, scorer.score());
    }
}
