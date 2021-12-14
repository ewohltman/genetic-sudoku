#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use super::sudoku::Board;
use arrayvec::ArrayVec;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct NoSolutionFound<const N: usize, const M: usize> {
    pub next_generation: ArrayVec<Board<N>, M>,
}

impl<const N: usize, const M: usize> Display for NoSolutionFound<N, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<const N: usize, const M: usize> Error for NoSolutionFound<N, M> {}
