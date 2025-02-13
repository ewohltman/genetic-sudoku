#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use genetic_sudoku::sudoku::{Board, Row};
use rand::{Rng, SeedableRng, rng};
use rand_pcg::Pcg64Mcg;

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

fn bench_rng(c: &mut Criterion) {
    let mut rng = rng();

    c.bench_function("rng", |b| {
        b.iter(|| black_box(&mut rng).random::<f32>());
    });
}

fn bench_pcg64mcg(c: &mut Criterion) {
    let mut os_rng = rand::prelude::StdRng::from_os_rng();
    let mut rng = Pcg64Mcg::from_rng(&mut os_rng);

    c.bench_function("Pcg64Mcg", |b| {
        b.iter(|| black_box(&mut rng).random::<f32>());
    });
}

criterion_group!(
    benches,
    bench_count_row_duplicates,
    bench_count_box_duplicates,
    bench_rng,
    bench_pcg64mcg,
);
criterion_main!(benches);
