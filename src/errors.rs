use super::Row;
use std::error::Error;
use std::fmt;

#[derive(Clone)]
pub enum ErrorType {
    InvalidSize(usize),
    InvalidRows(Vec<InvalidRow>),
}

#[derive(Clone)]
pub struct InvalidSolution {
    pub error: ErrorType,
}

impl fmt::Debug for InvalidSolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.error {
            ErrorType::InvalidSize(size) => writeln!(f, "Invalid size: {}", size)?,
            ErrorType::InvalidRows(rows) => {
                writeln!(f, "Invalid rows:")?;

                for row in rows.iter() {
                    writeln!(f, "{}", row)?
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for InvalidSolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl Error for InvalidSolution {}

impl InvalidSolution {
    pub fn new(error_type: ErrorType) -> InvalidSolution {
        InvalidSolution { error: error_type }
    }
}

#[derive(Clone)]
pub struct InvalidRow {
    row: Row,
}

impl fmt::Debug for InvalidRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.row)
    }
}

impl fmt::Display for InvalidRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl Error for InvalidRow {}
