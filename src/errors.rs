#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use super::sudoku::Board;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result};

pub struct NoSolutionFound<const N: usize> {
    pub next_generation: Vec<Board<N>>,
}

impl<const N: usize> Debug for NoSolutionFound<N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self, f)
    }
}

impl<const N: usize> Display for NoSolutionFound<N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "no solution found")
    }
}

impl<const N: usize> Error for NoSolutionFound<N> {}
