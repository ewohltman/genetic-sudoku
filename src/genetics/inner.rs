use crate::sudoku::{Board, Row, box_size};
use rand::{RngExt, SeedableRng, distr::Uniform};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;
use std::array;

pub fn next_generation<const N: usize>(
    params: &super::GAParams,
    base: &Board<N>,
    mut population_scores: Vec<(Board<N>, u16)>,
) -> Vec<Board<N>> {
    population_scores.par_sort_by_key(|(_, score)| *score);

    // Capture the fittest individuals before selection truncates them so they
    // can be carried into the next generation unchanged (elitism).
    let elites: Vec<Board<N>> = population_scores
        .iter()
        .take(params.elitism)
        .map(|(board, _)| *board)
        .collect();

    let survivors = natural_selection(params, population_scores);

    let mut children: Vec<Board<N>> = survivors
        .par_chunks_exact(2)
        .flat_map(|parents| make_children::<N>(params, base, (parents[0], parents[1])))
        .collect();

    // Overwrite the first entries with the elites. Ordering within the
    // population is irrelevant (fitness is recomputed next generation), so this
    // preserves the population size exactly.
    let elitism = elites.len().min(children.len());
    children[..elitism].copy_from_slice(&elites[..elitism]);

    children
}

fn natural_selection<const N: usize>(
    params: &super::GAParams,
    mut population_scores: Vec<(Board<N>, u16)>,
) -> Vec<Board<N>> {
    population_scores.truncate(params.num_survivors);

    population_scores
        .into_iter()
        .map(|(survivor, _)| survivor)
        .collect()
}

fn make_children<const N: usize>(
    params: &super::GAParams,
    base: &Board<N>,
    parents: (Board<N>, Board<N>),
) -> Vec<Board<N>> {
    let Board(parent_x) = parents.0;
    let Board(parent_y) = parents.1;
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");
    let values_range = Uniform::new_inclusive(1, max_digit).expect("invalid value range");
    let mutation_rate = f64::from(params.mutation_rate);

    (0..params.num_children_per_parent_pairs)
        .into_par_iter()
        .map_init(
            || Pcg64Mcg::from_rng(&mut rand::rng()),
            |rng, _| {
                let mut child = Board(array::from_fn(|i| {
                    let Row(x_values) = parent_x[i];
                    let Row(y_values) = parent_y[i];

                    // One draw supplies the row's inheritance coin flips: bit j
                    // chooses which parent column j comes from (N ≤ 64, as
                    // Scorer's u64 bitmask already requires).
                    let inherit: u64 = rng.random();

                    Row(array::from_fn(|j| {
                        if rng.random_bool(mutation_rate) {
                            rng.sample(values_range)
                        } else if (inherit >> j) & 1 == 1 {
                            x_values[j]
                        } else {
                            y_values[j]
                        }
                    }))
                }));

                if params.local_search_passes > 0 {
                    // Overlay the puzzle's fixed cells so local search scores
                    // against the real board rather than mutated junk, then
                    // greedily improve the free cells.
                    child = base.overlay(&child);
                    local_search::<N>(&mut child, base, params.local_search_passes, rng);
                }

                child
            },
        )
        .collect()
}

/// Greedily improves the non-fixed cells of `board` in place.
///
/// Runs up to `passes` sweeps; each sweep sets every non-fixed cell to the
/// digit that minimizes that cell's local conflicts (duplicates within its row,
/// column, and box), breaking ties uniformly at random. Fixed cells (non-zero
/// in `base`) are never modified. Every move is non-increasing in conflicts, so
/// fitness never worsens.
fn local_search<const N: usize>(
    board: &mut Board<N>,
    base: &Board<N>,
    passes: usize,
    rng: &mut Pcg64Mcg,
) {
    let max_digit = u8::try_from(N).expect("digit size exceeds 255");

    // Bits 1..=N: the candidate digits.
    let digits_mask: u64 = ((1 << N) - 1) << 1;

    for _ in 0..passes {
        for i in 0..N {
            for j in 0..N {
                if base.0[i].0[j] != 0 {
                    continue; // Fixed cell: leave the given value untouched.
                }

                let (row, column, sub_box) = conflict_masks::<N>(board, i, j);

                // Digits absent from all three groups have zero marginal cost
                // — the minimum possible. Pick one uniformly at random to keep
                // the tie-break unbiased.
                let zero_cost = !(row | column | sub_box) & digits_mask;

                if zero_cost != 0 {
                    let nth = rng.random_range(0..zero_cost.count_ones());

                    board.0[i].0[j] = nth_set_bit(zero_cost, nth);
                    continue;
                }

                let mut best_digit = board.0[i].0[j];
                let mut best_cost = u16::MAX;
                let mut ties = 0u32;

                for digit in 1..=max_digit {
                    let cost = mask_cost(row, column, sub_box, digit);

                    if cost < best_cost {
                        best_cost = cost;
                        best_digit = digit;
                        ties = 1;
                    } else if cost == best_cost {
                        ties += 1;

                        // Reservoir sampling: keep each tied digit with equal
                        // probability without collecting them.
                        if rng.random_range(0..ties) == 0 {
                            best_digit = digit;
                        }
                    }
                }

                board.0[i].0[j] = best_digit;
            }
        }
    }
}

/// Returns the position of the `n`th (0-indexed) set bit in `mask`.
///
/// # Panics
///
/// Panics if `mask` has fewer than `n + 1` set bits.
fn nth_set_bit(mut mask: u64, n: u32) -> u8 {
    for _ in 0..n {
        mask &= mask - 1; // Clear the lowest set bit.
    }

    assert!(mask != 0, "mask has fewer than n + 1 set bits");

    u8::try_from(mask.trailing_zeros()).expect("bit position exceeds 255")
}

/// Returns bitmasks of the digits present in the row, column, and box of
/// `(i, j)`, excluding the cell itself (bit `d` set = digit `d` present).
///
/// Built once per cell, they make the marginal cost of any candidate digit an
/// O(1) lookup via [`mask_cost`] instead of an O(N) scan per digit.
fn conflict_masks<const N: usize>(board: &Board<N>, i: usize, j: usize) -> (u64, u64, u64) {
    let box_size = box_size(N);
    let mut row: u64 = 0;
    let mut column: u64 = 0;
    let mut sub_box: u64 = 0;

    for k in 0..N {
        if k != j {
            row |= 1 << board.0[i].0[k];
        }

        if k != i {
            column |= 1 << board.0[k].0[j];
        }
    }

    let box_row = (i / box_size) * box_size;
    let box_col = (j / box_size) * box_size;

    for r in box_row..box_row + box_size {
        for c in box_col..box_col + box_size {
            if r != i || c != j {
                sub_box |= 1 << board.0[r].0[c];
            }
        }
    }

    (row, column, sub_box)
}

/// Returns the marginal fitness cost of placing `digit` in a cell whose group
/// occupancy is described by the [`conflict_masks`] `row`, `column`, and
/// `sub_box`: one point for each group that already contains `digit`.
///
/// This mirrors [`Board::fitness`](crate::sudoku::Board)'s per-group duplicate
/// count (each group contributes at most one duplicate for `digit` when the
/// cell is added), so greedily minimizing it over a single cell can never
/// increase the board's overall fitness.
fn mask_cost(row: u64, column: u64, sub_box: u64, digit: u8) -> u16 {
    u16::from((row >> digit) & 1 != 0)
        + u16::from((column >> digit) & 1 != 0)
        + u16::from((sub_box >> digit) & 1 != 0)
}

#[cfg(test)]
mod tests {
    use super::{conflict_masks, local_search, mask_cost, next_generation, nth_set_bit};
    use crate::genetics::GAParams;
    use crate::sudoku::{Board, Row};
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;

    // A base puzzle where only the four corners are fixed.
    const BASE: Board<4> = Board([
        Row([1, 0, 0, 4]),
        Row([0, 0, 0, 0]),
        Row([0, 0, 0, 0]),
        Row([4, 0, 0, 1]),
    ]);

    #[test]
    fn test_next_generation_preserves_elite() {
        // The unique best board (score 0) must survive into the next generation.
        let solved = Board([
            Row([1, 2, 3, 4]),
            Row([3, 4, 1, 2]),
            Row([2, 1, 4, 3]),
            Row([4, 3, 2, 1]),
        ]);
        let junk = Board([Row([2; 4]), Row([2; 4]), Row([2; 4]), Row([2; 4])]);

        let params = GAParams::new(8, 0.5, 0.5, 1, 0);
        let mut population_scores: Vec<(Board<4>, u16)> = vec![(junk, 20); 7];
        population_scores.push((solved, 0));

        let next = next_generation::<4>(&params, &BASE, population_scores);

        assert!(
            next.contains(&solved),
            "elite board was not carried forward"
        );
    }

    #[test]
    fn test_local_search_never_increases_fitness() {
        let mut rng = Pcg64Mcg::seed_from_u64(42);

        // A deliberately conflict-heavy candidate over the free cells.
        let mut board = BASE.overlay(&Board([Row([2; 4]), Row([2; 4]), Row([2; 4]), Row([2; 4])]));
        let before = board.fitness();

        local_search::<4>(&mut board, &BASE, 1, &mut rng);

        assert!(
            board.fitness() <= before,
            "local search increased fitness: {before} -> {}",
            board.fitness()
        );
    }

    #[test]
    fn test_local_search_reduces_reducible_conflict() {
        let mut rng = Pcg64Mcg::seed_from_u64(7);

        // Every free cell set to 2 leaves many reducible conflicts.
        let mut board = BASE.overlay(&Board([Row([2; 4]), Row([2; 4]), Row([2; 4]), Row([2; 4])]));
        let before = board.fitness();
        assert!(before > 0);

        // Enough passes to reach a local optimum on this tiny board.
        local_search::<4>(&mut board, &BASE, 8, &mut rng);

        assert!(board.fitness() < before, "local search made no progress");
    }

    #[test]
    fn test_local_search_leaves_fixed_cells_untouched() {
        let mut rng = Pcg64Mcg::seed_from_u64(1);

        let mut board = BASE.overlay(&Board([Row([3; 4]), Row([3; 4]), Row([3; 4]), Row([3; 4])]));

        local_search::<4>(&mut board, &BASE, 4, &mut rng);

        for i in 0..4 {
            for j in 0..4 {
                if BASE.0[i].0[j] != 0 {
                    assert_eq!(
                        board.0[i].0[j], BASE.0[i].0[j],
                        "fixed cell ({i}, {j}) was modified"
                    );
                }
            }
        }
    }

    #[test]
    fn test_conflict_masks_marginal_cost() {
        // Placing `1` at (0, 1): row already has a 1 at (0, 0), the box shares
        // that same 1, and the column has no 1 -> row + box = 2.
        let board = Board([
            Row([1, 0, 0, 4]),
            Row([0, 0, 0, 0]),
            Row([0, 0, 0, 0]),
            Row([4, 0, 0, 1]),
        ]);

        let (row, column, sub_box) = conflict_masks::<4>(&board, 0, 1);

        assert_eq!(mask_cost(row, column, sub_box, 1), 2);
        assert_eq!(mask_cost(row, column, sub_box, 2), 0);
        // Placing `4` at (0, 1): row has a 4 at (0, 3) -> row only = 1.
        assert_eq!(mask_cost(row, column, sub_box, 4), 1);
    }

    #[test]
    fn test_nth_set_bit() {
        // 0b1010_0110: set bits at positions 1, 2, 5, 7.
        assert_eq!(nth_set_bit(0b1010_0110, 0), 1);
        assert_eq!(nth_set_bit(0b1010_0110, 1), 2);
        assert_eq!(nth_set_bit(0b1010_0110, 2), 5);
        assert_eq!(nth_set_bit(0b1010_0110, 3), 7);
    }
}
