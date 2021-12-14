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

fn bench_count_duplicates_map(bench: &mut Bencher) {
    bench.iter(|| BAD_BOARD.count_duplicates())
}

fn bench_count_duplicates_array(bench: &mut Bencher) {
    bench.iter(|| BAD_BOARD.count_duplicates_array())
}

benchmark_group!(
    benches,
    bench_count_duplicates_map,
    bench_count_duplicates_array
);
benchmark_main!(benches);
