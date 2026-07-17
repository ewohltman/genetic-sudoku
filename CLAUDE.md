# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`genetic-sudoku` is a Rust program that solves Sudoku puzzles using a multithreaded genetic algorithm, displaying progress in a terminal UI (ratatui).

## Commands

```bash
make build          # cargo build --release (default target)
make test           # cargo test
make fmt            # cargo fmt --all --check (check only, does not write)
make clippy         # cargo clippy --all-targets -- -D warnings
make bench          # cargo bench (criterion, full run — local only)
make bench-check    # cargo bench --no-run (compile check, used in CI)
make ci             # fmt + clippy + test + bench-check + build (mirrors GitHub Actions CI)
```

Run a single test: `cargo test test_board_fitness`

Run the app: `cargo run --release -- boards/default.txt` (quit with `q`, `Esc`, or Ctrl+C). Sample puzzles live in `boards/`.

## Architecture

The core loop lives in `src/main.rs`: read a board, generate an initial random population, then repeatedly call `genetics::run_simulation`, which returns `Ok(Board)` when a solution is found or `Err(NoSolutionFound)` carrying the `next_generation` (plus `best_board`/`best_score`) to feed back into the next iteration. Rendering (ratatui, using its re-exported crossterm) and keyboard polling are interleaved with this loop; the `--render` flag controls how often the TUI redraws, and `--restart` regenerates the population after N stagnant generations.

- `src/sudoku.rs` — `Board<const N: usize>` and `Row<N>` (fixed-size arrays, `Copy`). Board I/O (`read`), `overlay` (candidate solution over the puzzle's zero cells), and `fitness` (u16 sum of duplicate counts across rows, columns via transpose, and boxes; 0 = solved). `src/sudoku/inner.rs` has the bitmask-based `Scorer` used for duplicate counting.
- `src/genetics.rs` — `GAParams`, initial population generation, and `run_simulation` (rayon-parallel fitness evaluation). `src/genetics/inner.rs` implements selection/crossover/mutation (`next_generation`): sort by score, keep the top `selection_rate` fraction, pair survivors, and breed children with per-cell mutation/inheritance.
- `src/errors.rs` — `NoSolutionFound<N>`, doubling as the carrier of the next generation between iterations.

Board size is a const generic `N` throughout; the binary fixes it via `BOARD_SIZE` in `src/main.rs` (9). Only perfect-square sizes are supported (box size is derived via `isqrt`; non-squares panic), and `Board::read` can only parse single-digit cells (N ≤ 9).

## Conventions

- Strict lint policy: `clippy::all`, `clippy::pedantic`, `clippy::nursery`, `clippy::cargo` at `warn` via the `[lints.clippy]` table in `Cargo.toml`; CI runs clippy with `-D warnings`, so it must pass cleanly. `clippy.toml` allowlists duplicate transitive crates — refresh it when the lockfile changes.
- Public functions are consistently annotated `#[inline]` and `#[must_use]`/`# Errors`/`# Panics` doc sections — follow this pattern for new APIs.
- RNG pattern: `Pcg64Mcg::from_rng(&mut rand::rng())`, constructed once outside hot loops (via rayon `map_init` in parallel code). rand 0.10: the extension trait is `RngExt`.
- Unit tests live in `#[cfg(test)]` modules within source files (use `tempfile` for file fixtures); benchmarks in `benches/benches.rs` (criterion, `harness = false`).
