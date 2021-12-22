#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use super::sudoku::Board;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct NoSolutionFound<const N: usize> {
    pub next_generation: Vec<Board<N>>,
}

impl<const N: usize> Display for NoSolutionFound<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<const N: usize> Error for NoSolutionFound<N> {}
