# genetic-sudoku

`genetic-sudoku` is a Rust program designed to solve Sudoku puzzles using a
multithreaded genetic algorithm.

The program is designed to not output solutions. When a solution is found, it
will output the following metrics and loop to generate a new random population
to start again with:

* The current generation the solution was found in
* The duration of time it took to find the solution
* The average generation a solution is found in
* The average duration it takes to find a solution

The Sudoku puzzle board is modeled as a matrix. Values of 0 represent empty
cells in the puzzle, and non-zero values are the numbers given in the puzzle.

A default puzzle is built in that looks like this:

```rust
#[inline]
#[must_use]
pub const fn default() -> Board<9> {
    Board([
        Row([0, 0, 4, 0, 5, 0, 0, 0, 0]),
        Row([9, 0, 0, 7, 3, 4, 6, 0, 0]),
        Row([0, 0, 3, 0, 2, 1, 0, 4, 9]),
        Row([0, 3, 5, 0, 9, 0, 4, 8, 0]),
        Row([0, 9, 0, 0, 0, 0, 0, 3, 0]),
        Row([0, 7, 6, 0, 1, 0, 9, 2, 0]),
        Row([3, 1, 0, 9, 7, 0, 2, 0, 0]),
        Row([0, 0, 9, 1, 8, 2, 0, 0, 3]),
        Row([0, 0, 0, 0, 6, 0, 1, 0, 0]),
    ])
}
```

It's instantiated in `main.rs`:

```rust
const BASE: Board<9> = sudoku::default();
```

The `Al Escargot` Sudoku puzzle is also available:

```rust
#[inline]
#[must_use]
pub const fn al_escargot() -> Board<9> {
    Board([
        Row([1, 0, 0, 0, 0, 7, 0, 9, 0]),
        Row([0, 3, 0, 0, 2, 0, 0, 0, 8]),
        Row([0, 0, 9, 6, 0, 0, 5, 0, 0]),
        Row([0, 0, 5, 3, 0, 0, 9, 0, 0]),
        Row([0, 1, 0, 0, 8, 0, 0, 0, 2]),
        Row([6, 0, 0, 0, 0, 4, 0, 0, 0]),
        Row([3, 0, 0, 0, 0, 0, 0, 1, 0]),
        Row([0, 4, 0, 0, 0, 0, 0, 0, 7]),
        Row([0, 0, 7, 0, 0, 0, 3, 0, 0]),
    ])
}
```

To use it, `main.rs` will need to be updated:

```rust
const BASE: Board<9> = sudoku::al_escargot();
```

The genetic algorithm is designed to work like so:

* Generate a random population of 100 potential solutions
* For each potential solution:
  * "Overlay" the potential solution on top of the base board we're
looking to solve for by only replacing the base board's 0 cells
  * Evaluate the "fitness" of the potential solution by counting the number of
duplicated numbers in each row and column, then summing them to produce a
score. The lower the score, the better the solution with a fitness score of 0
being a valid solution to the puzzle
* With all the potential solution fitness scores calculated:
  * Sort them and apply "natural selection" to filter out only the top 50%
  * Of the remaining 50%, group them into 25 pairs
  * Have each pair produce 4 children to create the next generation's
population of 100 potential solutions
    * When each child is created, for each value there is a 5% chance to
randomly "mutate" and generate a whole new value
    * The other 95% of the time, there is a 50% chance to "inherit" the value
from one parent, and a 50% chance to "inherit" from the other parent
* Loop this process until a valid solution is found
