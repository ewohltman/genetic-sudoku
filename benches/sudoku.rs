#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use genetic_sudoku::sudoku::{Board, Row};

const BAD_BOARD: Board<4> = Board([
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
]);

fn bench_count_row_duplicates(c: &mut Criterion) {
    c.bench_function("count_row_duplicates", |b| {
        b.iter(|| black_box(BAD_BOARD).count_row_duplicates());
    });
}

fn bench_count_box_duplicates(c: &mut Criterion) {
    c.bench_function("count_box_duplicates", |b| {
        b.iter(|| black_box(BAD_BOARD).count_box_duplicates());
    });
}

criterion_group!(
    benches,
    bench_count_row_duplicates,
    bench_count_box_duplicates
);
criterion_main!(benches);
