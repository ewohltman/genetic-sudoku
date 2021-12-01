#![warn(
clippy::all,
// clippy::restriction,
clippy::pedantic,
clippy::nursery,
clippy::cargo,
)]

use super::sudoku::Row;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub enum ErrorType {
    InvalidSize(usize),
    InvalidRows(Vec<InvalidRow>),
}

#[derive(Clone)]
pub struct InvalidSolution {
    pub error: ErrorType,
}

impl InvalidSolution {
    #[must_use]
    pub const fn new(error_type: ErrorType) -> Self {
        Self { error: error_type }
    }
}

impl Debug for InvalidSolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.error {
            ErrorType::InvalidSize(size) => writeln!(f, "Invalid size: {}", size)?,
            ErrorType::InvalidRows(rows) => {
                writeln!(f, "Invalid rows:")?;

                for row in rows.iter() {
                    writeln!(f, "{}", row)?;
                }
            }
        }

        Ok(())
    }
}

impl Display for InvalidSolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Error for InvalidSolution {}

#[derive(Clone)]
pub struct InvalidRow {
    row: Row,
}

impl Debug for InvalidRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.row)
    }
}

impl Display for InvalidRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Error for InvalidRow {}
