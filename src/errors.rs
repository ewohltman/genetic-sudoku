use super::sudoku::Board;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct NoSolutionFound<const N: usize> {
    pub next: Vec<Board<N>>,
}

impl<const N: usize> Display for NoSolutionFound<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<const N: usize> Error for NoSolutionFound<N> {}
