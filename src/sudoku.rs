mod inner;

use crate::sudoku::inner::Scorer;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Board<const N: usize>(pub [Row<N>; N]);

impl<const N: usize> Board<N> {
    /// Overlays the given `overlay` on top of `self`.
    ///
    /// Returns a Result containing a new Board with the provided `overlay`
    /// on top of `self`.
    ///
    /// # Arguments
    ///
    /// * `overlay` - A Board to overlay on top of `self`
    /// ```
    #[inline]
    #[must_use]
    pub(crate) fn overlay(&self, overlay: &Self) -> Self {
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
    pub(crate) fn count_row_duplicates(&self) -> u8 {
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
    pub(crate) fn count_box_duplicates(&self) -> u8 {
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
        let format_error_rows = || {
            Error::new(
                ErrorKind::InvalidData,
                "malformed sudoku board: invalid number of rows",
            )
        };
        let format_error_columns = || {
            Error::new(
                ErrorKind::InvalidData,
                "malformed sudoku board: invalid number of columns",
            )
        };
        let format_error_char = || {
            Error::new(
                ErrorKind::InvalidData,
                "malformed sudoku board: invalid character",
            )
        };

        for r in &mut board_array {
            let line = lines.next().ok_or_else(format_error_rows)?;
            let mut row = Row::default();
            let mut parsed: usize = 0;

            for (i, ch) in line.chars().enumerate() {
                if i >= N {
                    return Err(format_error_columns());
                }

                #[allow(clippy::cast_possible_truncation)]
                {
                    let digit = ch.to_digit(10).ok_or_else(format_error_char)?;
                    row.0[i] = digit as u8;
                }

                parsed += 1;
            }

            if parsed != N {
                return Err(format_error_columns());
            }

            *r = row;
        }

        if lines.next().is_some() {
            return Err(format_error_rows());
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

#[cfg(test)]
mod tests {
    use super::{Board, Row};
    use std::fmt;
    use std::io::{Error, ErrorKind, Write};
    use std::path::PathBuf;
    use std::vec::Vec;
    use std::{env, fs};

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

    struct TempFile {
        file: fs::File,
        path: PathBuf,
    }

    impl TempFile {
        fn new(file_name: &str) -> Self {
            let dir = env::temp_dir();
            let path = dir.as_path().join(file_name);

            TempFile {
                file: fs::File::create(&path).unwrap(),
                path,
            }
        }

        fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<(), Error> {
            self.file.write_fmt(fmt)
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            fs::remove_file(&self.path).unwrap();
        }
    }

    #[test]
    fn test_board_new() {
        const N: usize = 4;

        let mut expected_rows: Vec<Row<N>> = Vec::with_capacity(N);

        for _ in 0..N {
            expected_rows.push(Row([0; N]));
        }

        let expected = Board::<N>(expected_rows.try_into().unwrap());
        let actual = Board::<N>([
            Row::<N>::default(),
            Row::<N>::default(),
            Row::<N>::default(),
            Row::<N>::default(),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_board_overlay() {
        const N: usize = 4;

        let mut overlay_rows: Vec<Row<N>> = Vec::with_capacity(N);
        let mut board_rows: Vec<Row<N>> = Vec::with_capacity(N);

        for _ in 0..N - 1 {
            board_rows.push(Row([0; N]));
            overlay_rows.push(Row([1; N]));
        }

        board_rows.push(Row([9; N])); // Row of numbers to not be replaced.
        overlay_rows.push(Row([1; N]));

        let board = Board::<N>(board_rows.try_into().unwrap());
        let overlay = Board::<N>(overlay_rows.try_into().unwrap());

        // expected:
        // 1111
        // 1111
        // 1111
        // 9999

        let expected = Board::<N>([Row([1; N]), Row([1; N]), Row([1; N]), Row([9; N])]);
        let actual = board.overlay(&overlay);

        assert_eq!(actual, expected);
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

    #[test]
    fn test_board_read_valid() {
        const N: usize = 4;

        let mut file = TempFile::new("valid_board.txt");
        let mut expected_rows: Vec<Row<N>> = Vec::with_capacity(N);

        for _ in 0..N {
            writeln!(file, "{}", "0".repeat(N)).unwrap();
            expected_rows.push(Row([0; N]));
        }

        let actual = Board::<N>::read(&file.path).unwrap();
        let expected = Board(expected_rows.try_into().unwrap());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_board_read_valid_larger_size() {
        const N: usize = 9;

        let mut file = TempFile::new("larger_board.txt");
        let mut expected_rows: Vec<Row<N>> = Vec::with_capacity(N);

        for _ in 0..N {
            writeln!(file, "{}", "0".repeat(N)).unwrap();
            expected_rows.push(Row([0; N]));
        }

        let actual = Board::<N>::read(&file.path).unwrap();
        let expected = Board(expected_rows.try_into().unwrap());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_board_read_nonexistent_file() {
        const N: usize = 4;

        let dir = env::temp_dir();
        let file_path = dir.as_path().join("nonexistent.txt");

        let actual = Board::<N>::read(&file_path);

        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().kind(), ErrorKind::NotFound);
    }

    #[test]
    fn test_board_read_malformed_too_few_rows() {
        const N: usize = 4;

        let mut file = TempFile::new("too_few_rows_board.txt");

        for _ in 0..(N - 1) {
            writeln!(file, "{}", "0".repeat(N)).unwrap();
        }

        let actual = Board::<N>::read(&file.path);

        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_board_read_malformed_too_many_rows() {
        const N: usize = 4;

        let mut file = TempFile::new("too_many_rows_board.txt");

        for _ in 0..(N + 1) {
            writeln!(file, "{}", "0".repeat(N)).unwrap();
        }

        let actual = Board::<N>::read(&file.path);

        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_board_read_malformed_too_few_columns() {
        const N: usize = 4;

        let mut file = TempFile::new("too_few_columns_board.txt");

        for _ in 0..N {
            writeln!(file, "{}", "0".repeat(N - 1)).unwrap();
        }

        let actual = Board::<N>::read(&file.path);

        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_board_read_malformed_too_many_columns() {
        const N: usize = 4;

        let mut file = TempFile::new("too_many_columns_board.txt");

        for _ in 0..N {
            writeln!(file, "{}", "0".repeat(N + 1)).unwrap();
        }

        let actual = Board::<N>::read(&file.path);

        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_board_read_malformed_invalid_char() {
        const N: usize = 4;

        let mut file = TempFile::new("bad_char_board.txt");

        for _ in 0..N - 1 {
            writeln!(file, "{}", "0".repeat(N)).unwrap();
        }

        writeln!(file, "{}", "A".repeat(N)).unwrap();

        let actual = Board::<N>::read(&file.path);

        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_board_display() {
        const N: usize = 9;

        let board = Board::<N>([
            Row([0; N]),
            Row([0; N]),
            Row([0; N]),
            Row([0; N]),
            Row([0; N]),
            Row([0; N]),
            Row([0; N]),
            Row([0; N]),
            Row([0; N]),
        ]);

        let expected = "\
        0 0 0 | 0 0 0 | 0 0 0\n\
        0 0 0 | 0 0 0 | 0 0 0\n\
        0 0 0 | 0 0 0 | 0 0 0\n\
        ----------------------\n\
        0 0 0 | 0 0 0 | 0 0 0\n\
        0 0 0 | 0 0 0 | 0 0 0\n\
        0 0 0 | 0 0 0 | 0 0 0\n\
        ----------------------\n\
        0 0 0 | 0 0 0 | 0 0 0\n\
        0 0 0 | 0 0 0 | 0 0 0\n\
        0 0 0 | 0 0 0 | 0 0 0\n"
            .to_string();
        let actual = format!("{board}");

        assert_eq!(actual, expected);
    }
    #[test]
    fn test_row_default() {
        const N: usize = 4;

        let expected = Row([0, 0, 0, 0]);
        let actual = Row::<N>::default();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_row_display() {
        let row = Row([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let expected = "1 2 3 | 4 5 6 | 7 8 9".to_string();
        let actual = format!("{row}");

        assert_eq!(actual, expected);
    }
}
