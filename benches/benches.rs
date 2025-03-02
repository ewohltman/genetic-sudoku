#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use genetic_sudoku::genetics;
use genetic_sudoku::genetics::GAParams;
use genetic_sudoku::sudoku::{Board, Row};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng, rng};
use rand_pcg::Pcg64Mcg;
use std::time::Duration;

const BOARD_SIZE_9: usize = 9;

const POPULATION: usize = 100;

const SELECTION_RATE: f32 = 0.5;

const MUTATION_RATE: f32 = 0.05;

fn bench_board_fitness(c: &mut Criterion) {
    const FITNESS_BOARD: Board<BOARD_SIZE_9> = Board([
        Row([9, 9, 9, 9, 9, 9, 9, 9, 9]),
        Row([9, 9, 9, 9, 9, 9, 9, 9, 9]),
        Row([9, 9, 9, 9, 9, 9, 9, 9, 9]),
        Row([9, 9, 9, 9, 9, 9, 9, 9, 9]),
        Row([9, 9, 9, 9, 9, 9, 9, 9, 9]),
        Row([9, 9, 9, 9, 9, 9, 9, 9, 9]),
        Row([9, 9, 9, 9, 9, 9, 9, 9, 9]),
        Row([9, 9, 9, 9, 9, 9, 9, 9, 9]),
        Row([9, 9, 9, 9, 9, 9, 9, 9, 9]),
    ]);

    c.bench_function("board_fitness", |b| {
        b.iter(|| black_box(FITNESS_BOARD).fitness());
    });
}

fn bench_generate_initial_population(c: &mut Criterion) {
    let params = GAParams::new(POPULATION, SELECTION_RATE, MUTATION_RATE);

    c.bench_function("generate_initial_population", |b| {
        b.iter(|| genetics::generate_initial_population::<BOARD_SIZE_9>(&params));
    });
}

fn bench_run_simulation(c: &mut Criterion) {
    const BOARD_SIZE_4: usize = 4;

    const EASY_4: Board<BOARD_SIZE_4> = Board([
        Row([1, 0, 0, 4]),
        Row([0, 4, 1, 2]),
        Row([2, 0, 4, 3]),
        Row([4, 3, 0, 0]),
    ]);

    let params = GAParams::new(POPULATION, SELECTION_RATE, MUTATION_RATE);
    let population = genetics::generate_initial_population::<BOARD_SIZE_4>(&params);

    c.bench_function("run_simulation", |b| {
        b.iter(|| genetics::run_simulation::<BOARD_SIZE_4>(&params, &EASY_4, population.clone()));
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
    name = benches;
    config = Criterion::default();
    targets = bench_board_fitness,
    bench_generate_initial_population,
    bench_rng,
    bench_pcg64mcg
);
criterion_group!(
    name = benches_long;
    config = Criterion::default().measurement_time(Duration::from_secs(30));
    targets = bench_run_simulation
);
criterion_main!(benches, benches_long);
