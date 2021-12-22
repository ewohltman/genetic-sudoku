#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use genetic_sudoku::sudoku::{Board, Row};
use rand::rngs::OsRng;
use rand::{thread_rng, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

const BAD_BOARD: Board<4> = Board([
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
]);

fn bench_count_row_duplicates(c: &mut Criterion) {
    c.bench_function("count_row_duplicates", |b| {
        b.iter(|| black_box(BAD_BOARD.count_row_duplicates()));
    });
}

fn bench_count_box_duplicates(c: &mut Criterion) {
    c.bench_function("count_box_duplicates", |b| {
        b.iter(|| black_box(BAD_BOARD.count_box_duplicates()));
    });
}

fn bench_thread_rng(c: &mut Criterion) {
    let mut rng = thread_rng();

    c.bench_function("thread_rng", |b| {
        b.iter(|| black_box(rng.gen::<f32>()));
    });
}

fn bench_pcg64mcg(c: &mut Criterion) {
    let mut rng = Pcg64Mcg::from_rng(OsRng).unwrap();

    c.bench_function("Pcg64Mcg", |b| {
        b.iter(|| black_box(rng.gen::<f32>()));
    });
}

criterion_group!(
    benches,
    bench_count_row_duplicates,
    bench_count_box_duplicates,
    bench_thread_rng,
    bench_pcg64mcg,
);
criterion_main!(benches);
