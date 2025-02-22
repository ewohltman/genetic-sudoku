#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use genetic_sudoku::genetics;
use genetic_sudoku::genetics::{GAParams, MAX_POPULATION};
use genetic_sudoku::sudoku::{Board, Row};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng, rng};
use rand_pcg::Pcg64Mcg;

const BOARD_SIZE_4: usize = 4;

const BOARD_SIZE_9: usize = 9;

const GENERATION: usize = 0;

const POPULATION: usize = 100;

const SELECTION_RATE: f32 = 0.5;

const MUTATION_RATE: f32 = 0.05;

const BAD_BOARD: Board<4> = Board([
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
    Row([1, 2, 3, 4]),
]);

const EASY_4: Board<4> = Board([
    Row([1, 0, 0, 4]),
    Row([0, 4, 1, 2]),
    Row([2, 0, 4, 3]),
    Row([4, 3, 0, 0]),
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

fn bench_generate_initial_population(c: &mut Criterion) {
    let params = GAParams::new(POPULATION, SELECTION_RATE, MUTATION_RATE, None);

    c.bench_function("generate_initial_population", |b| {
        b.iter(|| genetics::generate_initial_population::<BOARD_SIZE_9, MAX_POPULATION>(&params));
    });
}

fn bench_run_simulation(c: &mut Criterion) {
    let params = GAParams::new(POPULATION, SELECTION_RATE, MUTATION_RATE, None);
    let population = genetics::generate_initial_population::<BOARD_SIZE_4, MAX_POPULATION>(&params);

    c.bench_function("run_simulation", |b| {
        b.iter(|| {
            genetics::run_simulation::<BOARD_SIZE_4, MAX_POPULATION>(
                &params,
                GENERATION,
                &EASY_4,
                population.clone(),
            )
        });
    });
}

fn bench_rng(c: &mut Criterion) {
    let mut rng = rng();

    c.bench_function("rng", |b| {
        b.iter(|| black_box(&mut rng).random::<f32>());
    });
}

fn bench_pcg64mcg(c: &mut Criterion) {
    let mut rng = Pcg64Mcg::from_rng(&mut StdRng::from_os_rng());

    c.bench_function("Pcg64Mcg", |b| {
        b.iter(|| black_box(&mut rng).random::<f32>());
    });
}

criterion_group!(
    benches,
    bench_count_row_duplicates,
    bench_count_box_duplicates,
    bench_generate_initial_population,
    bench_run_simulation,
    bench_rng,
    bench_pcg64mcg,
);
criterion_main!(benches);
