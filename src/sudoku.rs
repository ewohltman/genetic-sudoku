#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

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
        let mut board: [Row<N>; N] = [Row::default(); N];

        for (i, row) in self.0.iter().enumerate().take(N) {
            for (j, digit) in row.0.iter().enumerate().take(N) {
                board[i].0[j] = match *digit {
                    0 => overlay.0[i].0[j],
                    _ => self.0[i].0[j],
                };
            }
        }

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

        for row in &self.0 {
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
        // This could be a proper integer square root.
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

        let mut total_duplicates: u8 = 0;

        for row in (0..N).step_by(box_size) {
            for col in (0..N).step_by(box_size) {
                let mut scorer = Scorer::default();

                for r in &self.0[row..row + box_size] {
                    for value in r.0.iter().skip(col).take(box_size) {
                        scorer.check(*value);
                    }
                }

                total_duplicates += scorer.score();
            }
        }

        total_duplicates
    }

    fn transpose(&self) -> Self {
        let mut transposed: [Row<N>; N] = [Row::default(); N];

        for i in 0..N {
            for (j, row) in transposed.iter_mut().enumerate().take(N) {
                row.0[i] = self.0[i].0[j];
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
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let board = fs::read_to_string(path)?;
        let mut lines = board.lines();
        let mut board_array: [Row<N>; N] = [Row::default(); N];
        let format_error = || Error::new(ErrorKind::InvalidData, "malformed sudoku board");

        for r in &mut board_array {
            let line = lines.next().ok_or_else(format_error)?;
            let mut row = Row::default();

            for (i, ch) in line.chars().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                {
                    let digit = ch.to_digit(N as u32 + 1).ok_or_else(format_error)?;
                    row.0[i] = digit as u8;
                }
            }

            *r = row;
        }

        if lines.next().is_some() {
            return Err(format_error());
        }

        Ok(Self(board_array))
    }
}

impl<const N: usize> Display for Board<N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, row) in self.0.iter().enumerate() {
            writeln!(f, "{row}")?;

            if i != 0 && (i + 1) % 3 == 0 && i < self.0.len() - 1 {
                writeln!(f, "{}", "--".repeat(N + 2))?;
            }
        }

        Ok(())
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
        for (i, value) in self.0.iter().enumerate() {
            write!(f, "{value}")?;

            if i < self.0.len() - 1 {
                write!(f, " ")?;

                if i != 0 && (i + 1) % 3 == 0 {
                    write!(f, "| ")?;
                }
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
    #[inline]
    fn check(&mut self, digit: u8) {
        let bit = 1 << digit;

        if self.seen & bit != 0 {
            self.score += 1;
        } else {
            self.seen |= bit;
        }
    }

    #[inline]
    const fn score(self) -> u8 {
        self.score
    }
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

        // Since Scorer.seen is u64, it supports up to 49 before overflowing.
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
