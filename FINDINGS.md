# Parameter Tuning Study

A study to find good values for the three genetic-algorithm knobs exposed by the
`genetic-sudoku` binary — **population** (`-p`), **selection rate** (`-s`), and
**mutation rate** (`-m`) — on two puzzles of very different difficulty:

1. [`boards/default.txt`](#board-1-defaulttxt) — an easy, quickly-solvable board.
2. [`boards/al-escargot.txt`](#board-2-al-escargottxt) — the historically hard
   *Al Escargot* puzzle.

## TL;DR — Recommended values

The optimal settings are **opposite** for the two boards, which is the headline
result (see [Cross-board discussion](#cross-board-discussion)):

| Parameter | Default | `default.txt` | `al-escargot.txt` |
|-----------|---------|---------------|-------------------|
| `-p` / `--population`     | `100`  | **`75`** (50–100) | **`3000`** (2000–3500) |
| `-s` / `--selection-rate` | `0.5`  | **`0.5`** (0.4–0.5) | **`0.4`** (0.35–0.5) |
| `-m` / `--mutation-rate`  | `0.06` | **`0.06`** (default) | **`0.02`** (0.01–0.02) |

```bash
# default.txt — solves in ~0.2 s
target/release/genetic-sudoku -p 75 -s 0.5 -m 0.06 boards/default.txt

# al-escargot.txt — gets closest, but see caveat below
target/release/genetic-sudoku -p 3000 -s 0.4 -m 0.02 boards/al-escargot.txt
```

For `default.txt` the program's built-in defaults are already near-optimal — only
the population wants nudging down. For `al-escargot.txt` the defaults are poor
(mutation ~3× too high, population ~30× too small), yet even the tuned values
**never fully solve it** — see that section's caveat.

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
  the fixed cursor address `ESC[3;8H`. A run is **solved** the instant that cell
  becomes `0`; wall-clock time is recorded at that moment.
- Otherwise the run is killed at a **time budget** and the last-seen best score
  is recorded.

Validated against `boards/trivial.txt` (solves in ~0.07 s), so solve detection is
real. The generation counter shown in the TUI is published on a throttled
`bounded(1)` channel at ~30 fps, so exact generation counts are approximate and
are treated only as a rough diagnostic; **solve rate and solve time are the
reliable signals.**

### Metrics — different per board

Because the two boards live in different regimes, they are ranked differently:

- **`default.txt` (solvable):** ranked by **solve rate** (fraction of trials that
  reach score 0) and **median solve time**. Budget 15 s, 8–10 trials per cell.
- **`al-escargot.txt` (essentially unsolvable in budget):** it almost never
  reaches 0, so it is ranked by **mean best score reached within the budget**
  (lower = closer to solved). Budget 20 s, 3 trials per cell.

`--restart 0` (disabled) throughout the main sweeps to isolate the three rates;
`--restart` is evaluated separately where relevant.

---

## Board 1: `default.txt`

An easy board the GA solves reliably and fast — when its parameters are in the
right window. Both parameters that *help* Al Escargot (big population, low
mutation) actively *hurt* here.

### 1. Population sweep (`-s 0.5 -m 0.06`)

| population | 20 | 30 | 50 | 75 | 100 | 150 | 200 | 400 | 800 | 1500 |
|-----------:|---:|---:|---:|---:|----:|----:|----:|----:|----:|-----:|
| solve rate | 0% | 0% | **100%** | **100%** | 100% | 60% | 50% | 25% | 25% | 12% |
| median solve time (s) | — | — | 0.57 | **0.22** | 1.1 | 2.4 | 4.6 | — | — | — |

A clear **window optimum at population 50–100** (best throughput at ~75, ~0.2 s):

- **Too small (≤30):** the population loses diversity almost immediately and
  converges to junk (stuck at ~24–33 conflicts, 0% solved).
- **Too large (≥150):** premature convergence plus far fewer generations per
  second — solve rate falls off steadily (150 → 60%, 200 → 50%, 800 → 25%).

This is the **opposite** of Al Escargot, where bigger populations were strictly
better. On an easy board, a small population runs many more generations per
second and its extra genetic drift is enough to finish; a big population just
converges genetically and stalls.

### 2. Selection × mutation grid (`-p 100`)

Solve rate (median solve time in parentheses where solved):

| `-s` / `-m` | 0.02 | 0.06 | 0.12 |
|------------:|-----:|-----:|-----:|
| **0.3** | 0% | 88% (1.5 s) | 0% |
| **0.5** | 0% | **100% (1.15 s)** | 0% |
| **0.7** | 0% | 0% | 0% |

- **Mutation `0.06` (the default) is optimal.** `0.02` is *too low* here — the
  population converges to a near-solution (~5–9 conflicts) and can't take the
  final step; `0.12` is too high and never converges (~35–57, near random). This
  is the **exact opposite** of Al Escargot, where `0.02` was best.
- **Selection `0.5` is best; `0.3` also works (88%); `0.7` fails.** High selection
  keeps too much of the population (weak selection pressure / almost no elitism),
  so it never converges (stuck at ~40–57).

### Recommended for `default.txt`

`-p 75 -s 0.5 -m 0.06` → 100% solve rate, ~0.2 s median. Essentially the program
defaults with a slightly smaller population. The board is aptly named: the
built-in defaults (`-p 100 -s 0.5 -m 0.06`) already solve it 100% of the time.

---

## Board 2: `al-escargot.txt`

The hard case. Tuning drives the board from ~20 residual conflicts (defaults)
down to ~2–6, but **no tested setting ever fully solves it** (see caveat).

### 1. Population sweep (`-s 0.5 -m 0.06`, 20 s, 3 trials)

| population | 200 | 500 | 1000 | 2000 | 3500 | 5000 |
|-----------:|----:|----:|-----:|-----:|-----:|-----:|
| mean final score | 19.7 | 15.3 | 14.3 | 15.0 | **11.0** | 13.7 |

Bigger population helps up to ~3500 (one trial reached score **7**). Past that it
reverses — a 5000-member population fits fewer generations into the budget.

### 2. Selection × mutation grid (`-p 3500`, 20 s, 3 trials)

Mean final score (lower = better):

| `-s` / `-m` | 0.02 | 0.06 | 0.10 | 0.16 |
|------------:|-----:|-----:|-----:|-----:|
| **0.20** | 7.7 | 7.7 | 17.3 | 47.7 |
| **0.35** | **3.0** | 10.3 | 33.7 | 37.7 |
| **0.50** | 5.0 | 15.3 | 49.0 | 54.3 |
| **0.65** | 5.0 | 17.0 | 52.3 | 55.7 |

- **Mutation is the dominant lever, and low wins.** `0.02` beats every other value
  in every row; the default `0.06` is markedly worse; `≥0.10` is catastrophic (too
  much noise to converge — scores stay near a random ~55).
- **Best cell: `-s 0.35 -m 0.02` → mean 3.0** (one trial hit score **1**).

### 3. Fine low-mutation sweep (`-p 3500`, 20 s, 3 trials)

Mean final score:

| `-s` / `-m` | 0.005 | 0.01 | 0.02 | 0.03 |
|------------:|------:|-----:|-----:|-----:|
| **0.30** | 6.0 | 9.7 | 7.0 | 25.7 |
| **0.40** | 8.0 | 9.0 | **5.0** | 7.3 |
| **0.50** | 5.3 | 5.0 | 5.7 | 7.0 |

The `0.005–0.02` mutation range is a **broad plateau** (mean ~5–9; several single
trials reached **2**). Below ~0.01 gives no further gain; selection `0.35–0.5` is
a wide, forgiving optimum.

### 4. Does it ever solve? (long budgets & restart)

| config | budget | trials | solved | best score | ~generations |
|--------|-------:|-------:|-------:|-----------:|-------------:|
| `-p 3500 -s 0.4 -m 0.02` | 120 s | 2 | 0 | 6 | — |
| `-p 2000 -s 0.4 -m 0.02` | 120 s | 2 | 0 | 6 | — |
| `-p 1000 -s 0.4 -m 0.02 --restart 500` | 90 s | 3 | 0 | 6 | 300,000 |
| `-p 2000 -s 0.4 -m 0.02 --restart 800` | 90 s | 3 | 0 | 5 | 200,000 |
| `-p 3500 -s 0.35 -m 0.02 --restart 1000` | 90 s | 3 | 0 | **3** | 100,000 |

Neither more time nor population restarts (hundreds of independent fresh attempts)
produced a solution. The GA converges within a few *hundred* generations and the
remaining tens of thousands make no progress — deep premature convergence.

### Caveat — it doesn't fully solve

**Al Escargot was never actually solved** at any tested setting. With `--restart
0`, a converged, low-diversity population plus low mutation has no path from ~5
conflicts to 0; raising mutation to escape instead prevents convergence
altogether; `--restart` gives many independent attempts but each stalls at ~2–6
the same way. This is a limitation of the algorithm's operators on a maximally
hard puzzle, not of the parameter values. The recommended settings are what get
*closest, fastest*.

---

## Cross-board discussion

The two boards want **opposite** settings on two of the three knobs:

| Knob | `default.txt` (easy) | `al-escargot.txt` (hard) | Why |
|------|----------------------|--------------------------|-----|
| Population | **small** (50–100) | **large** (~3000) | Easy board: small pop = more generations/sec + enough drift to finish. Hard board: large pop = broader exploration to push residual conflicts down; small pop converges to junk. |
| Mutation | **0.06** (moderate) | **0.02** (low) | Easy board *is* solvable, so you need enough mutation to take the final step to 0. Hard board never reaches 0, so low mutation minimizes residual conflicts; higher just adds noise. |
| Selection | 0.5 | 0.4 | Both prefer a moderate 0.35–0.5 (real selection pressure). `0.7` fails on the easy board — too many survivors, no elitism. |

**The practical lesson: there is no single best parameter set — it depends on
puzzle difficulty.**
- For a board you expect to *solve*, keep a **small population** and **moderate
  mutation (~0.06)** so it converges all the way to zero quickly.
- For a board you only expect to get *close* on, use a **large population** and
  **low mutation (~0.02)** to minimize residual conflicts.

The program's built-in defaults (`-p 100 -s 0.5 -m 0.06`) are tuned for the
easy/solvable regime and work well on `default.txt`, but are a poor fit for a hard
puzzle like Al Escargot.

## Notes & caveats

- **Stochastic.** Single-run variance is roughly ±3–5 score (hard board) and
  affects individual solve times (easy board); treat within-plateau differences
  as ties and prefer the center of a good region over the single best cell.
- **Budget-dependent.** The hard-board population optimum (~3000) is tied to the
  ~20 s budget — larger populations need a longer budget to pull ahead. The
  mutation findings are robust across budgets.
- **`--restart` was excluded** from the main sweeps because a reset just before
  the timeout leaves a fresh-random (~55) score, adding endpoint noise. It is
  evaluated separately for the hard board above.
- All data was produced with the `target/release/genetic-sudoku` binary against
  the respective board file.
