# Parameter Tuning Study — `boards/al-escargot.txt`

A study to find good values for the three genetic-algorithm knobs exposed by the
`genetic-sudoku` binary — **population** (`-p`), **selection rate** (`-s`), and
**mutation rate** (`-m`) — on the historically hard *Al Escargot* puzzle.

## TL;DR — Recommended values

| Parameter | Default | Recommended | Notes |
|-----------|---------|-------------|-------|
| `-m` / `--mutation-rate`  | `0.06` | **`0.02`** (0.01–0.02) | The dominant knob. The default is 2–3× too high. `≥0.10` is catastrophic. |
| `-p` / `--population`     | `100`  | **`3000`** (2000–3500) | Bigger explores better; past ~3500 too few generations fit the time budget. |
| `-s` / `--selection-rate` | `0.5`  | **`0.4`** (0.35–0.5) | Broad, forgiving optimum. `0.35` was the single best cell. |

```bash
target/release/genetic-sudoku -p 3000 -s 0.4 -m 0.02 boards/al-escargot.txt
```

**Honest caveat:** *Al Escargot was never actually solved* by this GA at any
tested setting, even with 300,000 generations and population restarts over 90 s.
Tuning drives the board from ~20 residual conflicts (defaults) down to ~2–6, but
the crossover/mutation operators cannot bridge the last few conflicts on this
puzzle. See [Why it doesn't fully solve](#why-it-doesnt-fully-solve). The
recommended values are what get *closest, fastest*.

---

## Methodology

### The measurement problem

The binary is a **ratatui TUI with no headless/benchmark mode**. It never exits
on its own: on solving it displays the board and waits for a keypress; otherwise
it runs forever. So it cannot be timed with `timeout` + exit code.

The harness (`pty`-based, Python) works around this:

- Runs the binary in a pseudo-terminal with a fixed 40×120 window size (ratatui
  renders nothing into a 0-area terminal).
- Parses the ratatui cell-diff byte stream. The score value is always written at
  the fixed cursor address `ESC[3;8H` and the generation at `ESC[2;13H`. A run is
  **solved** the instant the score cell becomes `0`.
- Records wall-clock time + generation on solve; otherwise kills the run at a
  **time budget** and records the last-seen best score.

Validated against `boards/trivial.txt` (solves in ~0.07 s, generation 1–2), so
solve detection is real — the zero solves on Al Escargot are genuine, not a
harness blind spot.

### Optimization metric

Because Al Escargot essentially never reaches score 0 within budget, ranking by
solve time is useless. Instead the primary metric is:

> **Mean final best score reached within a fixed wall-clock budget** (across
> trials; `0` = solved). Lower is better.

Score = the app's fitness = count of duplicate values across rows, columns, and
boxes. This is a smooth, discriminating signal that ranks parameter sets even
when none of them solve. Unless noted, budget = **20 s**, **3 trials** per cell.
`--restart 0` (disabled) throughout the main sweeps to isolate the three rates.

---

## Results

### 1. Population sweep (`-s 0.5 -m 0.06`, 20 s, 3 trials)

| population | 200 | 500 | 1000 | 2000 | 3500 | 5000 |
|-----------:|----:|----:|-----:|-----:|-----:|-----:|
| mean final score | 19.7 | 15.3 | 14.3 | 15.0 | **11.0** | 13.7 |

Bigger population helps a lot up to ~3500 (one trial there reached score **7**).
Past that it reverses — a 5000-member population fits fewer generations into the
budget. **Population is the second most important knob.**

### 2. Selection × mutation grid (`-p 3500`, 20 s, 3 trials)

Mean final score (lower = better):

| `-s` ＼ `-m` | 0.02 | 0.06 | 0.10 | 0.16 |
|------------:|-----:|-----:|-----:|-----:|
| **0.20** | 7.7 | 7.7 | 17.3 | 47.7 |
| **0.35** | **3.0** | 10.3 | 33.7 | 37.7 |
| **0.50** | 5.0 | 15.3 | 49.0 | 54.3 |
| **0.65** | 5.0 | 17.0 | 52.3 | 55.7 |

- **Mutation is the dominant lever.** `0.02` wins in *every* row; the default
  `0.06` is markedly worse; `≥0.10` is catastrophic — too much random noise for
  the population to ever converge (scores stay near a random ~55).
- **Best cell: `-s 0.35 -m 0.02` → mean 3.0** (one trial hit score **1**).

### 3. Fine low-mutation sweep (`-p 3500`, 20 s, 3 trials)

Mean final score:

| `-s` ＼ `-m` | 0.005 | 0.01 | 0.02 | 0.03 |
|------------:|------:|-----:|-----:|-----:|
| **0.30** | 6.0 | 9.7 | 7.0 | 25.7 |
| **0.40** | 8.0 | 9.0 | **5.0** | 7.3 |
| **0.50** | 5.3 | 5.0 | 5.7 | 7.0 |

The `0.005–0.02` mutation range is a **broad plateau** (mean ~5–9; several
single trials reached **2**). Below ~0.01 gives no further gain; 0.02 is a safe,
robust choice. Selection `0.35–0.5` is likewise a wide, forgiving optimum.

### 4. Generation throughput (20 s)

| population | generations in 20 s | gens/sec |
|-----------:|--------------------:|---------:|
| 1000 | ~80,000 | ~4000 |
| 2000 | ~50,000 | ~2500 |
| 3500 | ~30,000 | ~1500 |

Generations are extremely cheap (small board, rayon-parallel). The GA converges
within a few *hundred* generations and the remaining tens of thousands make **no
progress** — classic premature convergence.

### 5. Long-budget & restart attempts (does it ever solve?)

| config | budget | trials | solved | best score | generations |
|--------|-------:|-------:|-------:|-----------:|------------:|
| `-p 3500 -s 0.4 -m 0.02` | 120 s | 2 | 0 | 6 | — |
| `-p 2000 -s 0.4 -m 0.02` | 120 s | 2 | 0 | 6 | — |
| `-p 1000 -s 0.4 -m 0.02 --restart 500` | 90 s | 3 | 0 | 6 | ~300,000 |
| `-p 2000 -s 0.4 -m 0.02 --restart 800` | 90 s | 3 | 0 | 5 | ~200,000 |
| `-p 3500 -s 0.35 -m 0.02 --restart 1000` | 90 s | 3 | 0 | **3** | ~100,000 |

Neither more time nor population restarts (hundreds of independent fresh
attempts) produced a solution. More time is largely *wasted* once the population
converges.

---

## Why it doesn't fully solve

- With `--restart 0`, a converged, low-diversity population + low mutation has no
  path from ~5 conflicts to 0 — the operators can't fix the last few deeply
  constrained cells. Extra generations are wasted (a 120 s run scored no better
  than a 20 s run, and one even regressed to 19 by drifting).
- Raising mutation to escape the local optimum instead prevents convergence
  altogether (`-m ≥ 0.10` never gets close).
- `--restart` gives many independent attempts, but each attempt stalls at ~2–6
  the same way, so re-seeding alone didn't land a 0 either. It slightly improves
  *consistency* and had the single best result (score 3), so it is the only
  lever with any shot at eventually solving — but it is unreliable here.

This is a limitation of the algorithm/operators on a maximally hard puzzle, not
of the parameter values. The tuning above is what minimizes residual conflicts.

## Notes & caveats

- **Budget-dependent.** The population optimum (~3000) is tied to the ~20 s
  budget: larger populations do fewer generations per second and need a longer
  budget to pull ahead. The **mutation finding (~0.02) is robust** across all
  budgets tested and is the highest-impact change regardless.
- **`--restart` was excluded** from the main sweeps because a reset just before
  the timeout leaves a fresh-random (~55) score, adding endpoint noise. It is
  evaluated separately above.
- **Stochastic.** 3 trials/cell; single-run variance is roughly ±3–5 score, so
  treat within-plateau differences as ties and prefer the center of a good
  region (`-p 3000 -s 0.4 -m 0.02`) over chasing the single best cell.
- Harness and raw per-phase JSON outputs were produced with the
  `target/release/genetic-sudoku` binary against `boards/al-escargot.txt`.
