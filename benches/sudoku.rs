#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#[macro_use]
extern crate bencher;

use bencher::Bencher;
use genetic_sudoku::sudoku::{Board, Row};

const BAD_BOARD: Board<4> = Board([
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
]);

fn bench_count_duplicates(bench: &mut Bencher) {
    bench.iter(|| BAD_BOARD.count_duplicates());
}

benchmark_group!(benches, bench_count_duplicates);
benchmark_main!(benches);
