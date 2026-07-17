# Modernization Plan

A phased roadmap for updating, modernizing, and improving `genetic-sudoku`.
Each phase is independently landable: after every phase, `make ci` must pass
and the binary must solve a board end-to-end
(`cargo run --release -- boards/default.txt`, quit with `q`).

Baseline as of 2026-07: edition 2024, rustc 1.97, clean working tree, CI green.

## Phase 1 — Tooling, CI, and packaging hygiene

No behavior changes. Hardening the gates first means every later phase is
validated by a stricter CI.

### `Cargo.toml`

- Replace `license-file = "LICENSE"` with the SPDX field `license = "MIT"`
  (the `LICENSE` file stays).
- Add `rust-version = "1.85"` (edition 2024 floor; rand 0.10's MSRV). Bump
  only if a later phase demands it.
- Add a `[lints]` table and delete the repeated
  `#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]`
  headers from `src/lib.rs`, `src/main.rs`, `src/errors.rs`,
  `src/sudoku/inner.rs`, `src/genetics/inner.rs`, and `benches/benches.rs`:

  ```toml
  [lints.clippy]
  all = { level = "warn", priority = -1 }
  pedantic = { level = "warn", priority = -1 }
  nursery = { level = "warn", priority = -1 }
  cargo = { level = "warn", priority = -1 }
  ```

  Same policy, declared once, applied to all targets including benches.

### `Makefile`

- `clippy:` → `cargo clippy --all-targets -- -D warnings`. Today
  `cargo clippy` exits 0 on warnings, so CI does not actually enforce the
  lint policy.
- Add `bench-check: cargo bench --no-run` and use it in the `ci` target;
  keep `bench` for local full runs. Full criterion runs on shared CI runners
  are slow and statistically meaningless.
- Drop `clean` from the `ci` target (defeats caching; keep the standalone
  target).

### `.github/workflows/ci.yml`

- Add `Swatinem/rust-cache@v2` after the toolchain step.
- Swap the `make bench` step for `make bench-check`.
- Add a dependency-audit step (`cargo-deny` via
  `EmbarkStudios/cargo-deny-action` with a minimal `deny.toml`, or
  `actions-rust-lang/audit` for zero config).

### New `.github/dependabot.yml`

Weekly `cargo` and `github-actions` update ecosystems.

## Phase 2 — Dependency upgrades

Land as separate commits in this order (independent failure domains). After
each: `make ci`, then run the binary against `boards/default.txt` and
`boards/medium.txt`.

### 2a. Trivial bumps

`rayon = "1.12"`, `clap = "4.6"`. No code changes expected.

### 2b. criterion 0.5 → 0.8 (`benches/benches.rs`)

- criterion no longer exports `black_box`; use `std::hint::black_box`.
- `criterion_group!` / `criterion_main!` are unchanged.

### 2c. rand 0.9 → 0.10 + rand_pcg 0.9 → 0.10 (must move together)

`rand_pcg` 0.9 implements rand_core 0.9 traits, incompatible with rand 0.10.
Verify the exact renames against the rand 0.10 changelog when implementing;
expected impact on this repo:

- The `rand::Rng` extension trait is renamed `rand::RngExt`
  (`rand_core::RngCore` became `rand_core::Rng`). Update imports in
  `src/genetics.rs`, `src/genetics/inner.rs`, `benches/benches.rs`.
  `random_bool`, `sample`, `sample_iter` live on `RngExt`.
- `SeedableRng::from_os_rng` is removed. Replace the
  `Pcg64Mcg::from_rng(&mut StdRng::from_os_rng())` pattern with
  `Pcg64Mcg::from_rng(&mut rand::rng())` (thread-local, OS-seeded) and
  delete the `StdRng` intermediary entirely. Call sites:
  `generate_initial_population`, `make_children`, and the benches.
- Keep this commit a pure mechanical migration; the structural RNG fix
  (hoisting out of hot loops) lands in Phase 4.

### 2d. ratatui 0.29 → 0.30, drop direct crossterm dependency

- The `ratatui` facade crate keeps `init()` / `restore()` /
  `DefaultTerminal` / `Frame` / `Text::raw`, so the minimal migration is
  small. Recommended modernization: adopt the new
  `ratatui::run(|terminal| ...)` closure API, which handles init/restore and
  panic hooks — `main` becomes
  `simple_eyre::install()?; ratatui::run(|terminal| run(Args::parse(), terminal))`.
- Remove `crossterm` from `Cargo.toml`; change
  `use crossterm::event::{self, Event}` in `src/main.rs` to
  `use ratatui::crossterm::event::{self, Event}`. This eliminates the
  version-skew hazard between a direct crossterm dep and the one bundled
  with ratatui.
- While touching `should_quit`, only react to `key.is_press()` so
  key-release/repeat events don't trigger quit (Windows correctness).
- Confirm interactively: `q`, `Esc`, and `Ctrl+C` quit both mid-run and
  after a solution is displayed.

### Post-upgrade

Update `clippy.toml` `allowed-duplicate-crates` — the duplicate set under
`clippy::cargo` changes with the new lockfile.

## Phase 3 — Correctness fixes

### Widen the fitness score type (`u8` → `u16`)

Max possible fitness is 3·N·(N−1): 216 for N=9 (barely fits u8), **720 for
N=16 and 1800 for N=25 — u8 overflows** despite the code claiming 4/9/16/25
support. Change `fitness()`, `count_row_duplicates()`,
`count_box_duplicates()` in `src/sudoku.rs` to return `u16`, and thread
`u16` through `run_simulation`'s `Vec<(Board<N>, u16)>` in
`src/genetics.rs` and `next_generation` in `src/genetics/inner.rs`.
`Scorer` (`src/sudoku/inner.rs`) can stay `u8` internally (per-unit max is
N−1 ≤ 24); widen with `u16::from` at accumulation sites.

### Derive box size from N

Replace the `match N { 4 => 2, 9 => 3, 16 => 4, 25 => 5, _ => panic!(...) }`
in `count_box_duplicates` with `N.isqrt()` (stable since 1.84) plus an
assert that N is a perfect square. Use the same derived box size in
`Display for Board` and `Display for Row`, which currently hardcode `% 3`
separators and render N=4/16/25 boards incorrectly. Add a `Board<4>`
display test; the existing `Board<9>` test pins the 9×9 rendering.

### `GAParams::new` validation (`src/genetics.rs`)

- Fix stale docs: remove the nonexistent `restart` parameter, rename
  `frac_reduction` to `selection_rate`.
- Assert (documented under `# Panics`): `selection_rate` in `(0.0, 1.0]`,
  `mutation_rate` in `[0.0, 1.0]`, and `num_survivors >= 2`. Currently
  `--population 3 --selection-rate 0.5` panics with an opaque
  divide-by-zero.
- In `src/main.rs`, add clap `value_parser` ranges for the two rates and
  population so users get argument errors instead of panics.

### `Board::read` digit validation (`src/sudoku.rs`)

Reject parsed digits > N (0 = blank stays allowed). Currently
`Board::<4>::read` accepts a `9` and the GA chases an unsatisfiable target.
(The multi-character token format needed to *read* N=16/25 boards is
deferred to Phase 5.)

### Test infrastructure

Add `tempfile` as a dev-dependency and replace the hand-rolled `TempFile`
helper in `src/sudoku.rs` tests — its fixed filenames in `env::temp_dir()`
collide across concurrent checkouts/CI runs. `tempfile::NamedTempFile`
deletes the whole helper struct.

## Phase 4 — Performance and simplification

Benchmark protocol: before starting,
`cargo bench -- --save-baseline pre-perf`; after each change,
`cargo bench -- --baseline pre-perf`. Also wall-clock
`boards/medium.txt` and `boards/al-escargot.txt` with `--render 1000`
before/after.

### Hoist RNG construction out of hot loops (the big win)

A fresh OS-seeded RNG is currently constructed **per child** in
`make_children` (`src/genetics/inner.rs`) and **per row** in
`generate_initial_population` (`src/genetics.rs`) — OS entropy syscalls in
the innermost loops.

- `make_children`: use rayon's `map_init`:
  `(0..n).into_par_iter().map_init(|| Pcg64Mcg::from_rng(&mut rand::rng()), |rng, _| { ... })`
  — one PCG per rayon worker segment, reused across children.
- `generate_initial_population`: the loop is sequential; hoist a single
  `Pcg64Mcg` above both loops.

### Drop `arrayvec`

`std::array::from_fn` replaces every ArrayVec push-then-
`into_inner().unwrap()` dance:
`Row(std::array::from_fn(|_| rng.sample(values_range)))` in
`generate_initial_population`, and `from_fn(|j| ...)` over parent cells in
`make_children`. Removes a dependency and all the unwraps.

### Simplify `make_parents` (`src/genetics/inner.rs`)

Replace the enumerate/partition/double-collect/par-zip construction with
`survivors.par_chunks_exact(2).map(|pair| (pair[0], pair[1]))` (Board is
`Copy`). Return `impl ParallelIterator<Item = (Board<N>, Board<N>)>`,
deleting the `Zip<IntoIter<...>>` type gymnastics and the misnamed
`ScoreBoard` alias. Semantic note: with an odd survivor count the current
code drops a mid-ranked survivor via zip truncation; `chunks_exact` drops
the worst instead — a strict improvement.

### Minor

In `natural_selection`, replace `drain(..n)` + collect with `truncate(n)` +
map into boards (same output, simpler intent).

## Phase 5 — Optional features (priority-ordered, all skippable)

1. **Restart on stagnation.** Pure elitist truncation selection stalls on
   hard boards (`al-escargot.txt`) and spins forever on `unsat.txt`; the
   stale `restart` doc in `GAParams` suggests this existed once. Minimal
   design: add `pub best_score: u16` (optionally `best_board`) to
   `errors::NoSolutionFound` so `main.rs` can track stagnation — note
   `next_generation[0]` is *not* the best board today, it's a random child
   of the top pair, so the current "best board" display is already slightly
   dishonest. Add a `--restart <generations>` clap flag (0 = disabled); on
   stagnation, `main` regenerates the population. All logic stays in
   `main.rs` against the existing library API.
2. **Large-board input.** Whitespace-separated token format in
   `Board::read` for digits > 9, plus runtime board-size dispatch in
   `main.rs` (probe the file, monomorphize `run` for 4/9/16/25). Only
   worthwhile together; fine to defer indefinitely.
3. **`POLL_DURATION_RUNNING` → `Duration::ZERO`** — same non-blocking
   semantics as 1ns, clearer intent. One line in `src/main.rs`.

## Considered and rejected

- **Partial selection (`select_nth_unstable`) instead of full sort** — at
  the default population of 100 the parallel sort is noise, and parent
  pairing depends on rank order anyway; revisit only if benches at large
  populations show it matters.
- **Replacing `simple-eyre`** (with anyhow/color-eyre) — it works, is tiny,
  and isn't a maintenance risk worth the churn.
- **Restructuring the public module layout** — keep the
  `errors`/`genetics`/`sudoku` API shape; the only justified public
  signature changes are `fitness() -> u16` (latent overflow) and the
  additive `NoSolutionFound.best_score` field (Phase 5, optional).
- **"Fixing" the post-solution poll loop** — `event::poll(100ms)` blocks;
  it is not a spin loop, and the per-generation poll is one syscall dwarfed
  by a rayon fitness pass.

## Verification (every phase)

- `make ci` — after Phase 1 this genuinely gates fmt, clippy
  (`-D warnings`), tests, bench compilation, and the release build.
- `cargo run --release -- boards/default.txt` solves to score 0; `q`,
  `Esc`, and `Ctrl+C` exit cleanly.
- Phase 2d adds manual TUI interaction checks; Phase 4 adds the criterion
  baseline comparison and wall-clock spot checks.
