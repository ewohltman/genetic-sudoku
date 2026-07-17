# GA Parameter Tuning: Selection & Mutation Rates

Empirical study of the `--selection-rate` (s) and `--mutation-rate` (m)
parameters, measured with a headless harness driving `genetics::run_simulation`
directly (no TUI).

## Setup

- **Board:** `boards/default.txt` (45 empty cells)
- **Population:** 100 (the default)
- **Generation cap:** 150,000 per run (a run that hits the cap is counted as
  unsolved)
- **Reps:** 10 runs per parameter combination; medians reported
- **Metric:** solve reliability (solved/10) and median generations to a
  fitness-0 solution

## Recommendation

**`-s 0.15 -m 0.09`** is the sweet spot. Anything in `s ∈ [0.10, 0.25]` paired
with `m ≈ 0.09` performs comparably well.

- `-s 0.15 -m 0.09`: **10/10 solved, ~5,300 median generations (~0.23s)**
- Current `-s 0.25 -m 0.075`: 9/10 solved, ~21,000 generations (~0.89s)
- Repo defaults `-s 0.5 -m 0.06`: dead zone — usually fails to converge within
  150k generations

The recommended rates are roughly **4× faster and more reliable** than the
`-s 0.25 -m 0.075` currently in use.

## Refined sweep results

solved/10, median generations (`cap` = hit the 150k generation cap):

| s \ m | 0.06        | 0.075       | **0.09**        | 0.11        |
|-------|-------------|-------------|-----------------|-------------|
| 0.10  | 5/10, cap   | 7/10, 94k   | **10/10, 5.9k** | 10/10, 9.3k |
| 0.15  | 4/10, cap   | 9/10, 16k   | **10/10, 5.3k** ← best | 9/10, 50k |
| 0.25  | 2/10, cap   | 9/10, 21k   | **10/10, 10k**  | 0/10, cap   |

## Coarse sweep results (initial pass, 6 reps)

solved/6, median generations:

| s \ m | 0.02      | 0.05      | 0.075          | 0.11            |
|-------|-----------|-----------|----------------|-----------------|
| 0.10  | 0/6, cap  | 0/6, cap  | 3/6, cap       | **6/6, 12k**    |
| 0.25  | 0/6, cap  | 1/6, cap  | **6/6, 18k**   | 0/6, cap        |
| 0.50  | 0/6, cap  | 1/6, cap  | 2/6, cap       | 0/6, cap        |
| 0.80  | 0/6, cap  | 0/6, cap  | 0/6, cap       | 0/6, cap        |

## Why this shape

- **Mutation is the dominant knob.** The *effective* mutation rate is
  `m × free_cells`, because mutations landing on the puzzle's given cells are
  overwritten by `overlay`. With 45 free cells, `m = 0.09` ≈ 4 effective
  mutations per child — enough to keep escaping the local optima that Sudoku's
  deceptive fitness landscape is full of. At `m = 0.06` (~2.7 mutations/child)
  the population converges and mutation alone can't dig it back out, so runs
  stagnate.

- **Selection and mutation must balance — it's a ridge, not a plateau.** The
  viable region runs along a diagonal:
  - Stronger selection (`s = 0.10` keeps only 10 survivors) destroys diversity,
    so it needs *more* mutation to compensate (still solves at `m = 0.11`).
  - Weaker selection (`s = 0.25`) tolerates less noise and collapses to 0/10 at
    `m = 0.11`.

  Because it's a narrow ridge, small parameter changes flip between "solves in a
  fraction of a second" and "never solves."

- **Very weak selection is hopeless.** `s = 0.8` (keep 80% each generation) never
  solved at any mutation rate — there's almost no selection pressure.

## Caveats

- Measured on a single medium-difficulty board at population 100, n=10 reps. The
  ridge **shifts with population and puzzle difficulty**:
  - Larger populations hold more diversity and tolerate *lower* mutation rates.
  - Fewer givens → more free cells → the same per-child mutation target implies a
    *lower* m. For example, `al-escargot` has 58 free cells, suggesting
    `m ≈ 0.07` rather than 0.09.
- For genuinely hard puzzles (e.g. `al-escargot`), no rate setting makes a
  population-100 GA solve reliably. Pair these rates with `--restart` (a few
  thousand stagnant generations before regenerating) and a larger `--population`.

## Reproducing

The harness constructs `GAParams`, generates an initial population, and loops
`run_simulation` until it returns `Ok` or the generation cap is reached, timing
each run. It sweeps a selection × mutation grid and reports median generations
and solve counts per cell. Run it against any board file and population size to
re-tune for a different puzzle.
