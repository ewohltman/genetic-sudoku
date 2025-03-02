# genetic-sudoku

`genetic-sudoku` is a Rust program designed to solve Sudoku puzzles using a
multithreaded genetic algorithm.

## Usage

![demo](https://raw.githubusercontent.com/ewohltman/genetic-sudoku/main/demo/demo.gif)

```
Usage: genetic-sudoku [OPTIONS] <BOARD_PATH>

Arguments:
  <BOARD_PATH>  Path to board file

Options:
  -p, --population <POPULATION>
          Population per generation [default: 100]
  -s, --selection-rate <SELECTION_RATE>
          Fraction of population selected [default: 0.5]
  -m, --mutation-rate <MUTATION_RATE>
          Mutation rate as fraction [default: 0.06]
  -r, --render <RENDER>
          Number of generations between screen renders. Higher values give better computational performance
  -h, --help
          Print help
  -V, --version
          Print version
```

A Sudoku puzzle board file contains a textual matrix of values, with 0
representing empty cells in the puzzle, and non-zero values representing the
numbers given in the puzzle. The current source code deals only with 9×9 Sudoku
puzzles; the constant `BOARD_SIZE` in `src/main.rs` can be changed for other
puzzle sizes. The `boards/` directory contains a variety of puzzle boards.

The `--population`, `--selection-rate`, and `--mutation-rate` arguments specify
the parameters used in running the genetic algorithm described below. There are
sensible defaults for all of these. Note that the "fraction" arguments expect a
floating-point number between 0.0 and 1.0.

The optional `--render` argument specifies the number of generations between
screen renders. The higher the value the less often the screen will be rendered
with updates, allowing more CPU time for running the simulation.

## How It Works

* Generate a random population of potential solutions
* For each potential solution:
  * "Overlay" the potential solution on top of the base board we're looking to
    solve for by only replacing the base board's 0 cells
  * Evaluate the "fitness" of the potential solution by summing the number of
    duplicated values in each row, column, and box, to produce a fitness score.
    The lower the fitness score, the better the solution. A fitness score of 0
    is a valid solution to the puzzle
* With all the potential solution fitness scores calculated:
  * Sort them and apply "natural selection" to filter out only the top
    percentage determined by `--selection-rate`
  * Group the remaining potential solutions into pairs
  * Have each pair produce enough children to create the next generation's
    population of potential solutions
    * When each child is created, for each value there is a chance determined
      by `--mutation-rate` to randomly "mutate" and generate a whole new value
    * If the value is not mutated, there is a 50% chance to "inherit" the value
      from one parent and a 50% chance to "inherit" from the other parent
* Loop this process until a valid solution is found

## Acknowledgements

The Sudoku boards in `boards/mantere-koljonen` are from:

> Mantere, Timo and Janne Koljonen (2008). Solving and Analyzing Sudokus with
> Cultural Algorithms. In Proceedings of 2008 IEEE World Congress on
> Computational Intelligence (WCCI 2008), 1-6 June, Hong Kong, China, pages
> 4054-4061.

Made available [here](http://lipas.uwasa.fi/~timan/sudoku/).

Thanks much to the authors for collecting these.

The [Al Escargot](https://www.sudokuwiki.org/Escargot) board is by Arto Inkala.
