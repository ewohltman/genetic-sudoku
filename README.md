# genetic-sudoku

`genetic-sudoku` is a Rust program designed to solve Sudoku
puzzles using a multithreaded genetic algorithm.

By default, the program will output
* The current generation the solution was found in
* The duration of time it took to find the solution
* The solution

## How To Run

```
USAGE:
    genetic-sudoku [FLAGS] [OPTIONS] <BOARD>

FLAGS:
        --bench      runs program in benchmark mode
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --mutation <F>      mutation rate as fraction
        --population <N>    population per generation
        --restart <R>       number of generations to restart population
        --fraction <S>      fraction of population selected

ARGS:
    <BOARD>    board to solve
```

A Sudoku puzzle board file contains a textual matrix of
digits, with 0 representing empty cells in the puzzle, and
non-zero values representing the numbers given in the
puzzle. The current source code deals only with 9×9 Sudoku
puzzles; the constant `BOARD_SIZE` in `src/main.rs` can be
changed for other puzzle sizes. The `boards/` directory
contains a variety of puzzle boards.

The `--mutation`, `--population`, `--restart` and
`--fraction` arguments specify the parameters used in
running the genetic algorithm described below. There are
sensible defaults for all of these. Note that the "fraction"
arguments expect a floating-point number between 0.0 and 1.0.

The `--bench` argument causes the program to loop finding
solutions.  When a solution is found the program will not
output the solution, but will output the normal metrics, as
well as

* The average generation a solution is found in
* The average duration it takes to find a solution

It will then restart with a new random population.

## How It Works

The genetic algorithm is designed to work like so:

* Generate a random population of potential solutions, say 100 of them
* For each potential solution:
  * "Overlay" the potential solution on top of the base board we're
looking to solve for by only replacing the base board's 0 cells
  * Evaluate the "fitness" of the potential solution by
    counting the number of duplicated numbers in each row
    and column, then summing them to produce a score. The
    lower the score, the better the solution with a fitness
    score of 0 being a valid solution to the puzzle
* With all the potential solution fitness scores calculated:
  * Sort them and apply "natural selection" to filter out
    only the top percentage, say 50%
  * Group the remaining candidates into pairs
  * Have each pair produce enough children to create the next generation's
    population of potential solutions
    * When each child is created, for each value there is a
      chance, say 5%, to randomly "mutate" and generate a
      whole new value
    * The rest of the time, there is a 50% chance to "inherit" the value
      from one parent, and a 50% chance to "inherit" from the other parent
* Loop this process until a valid solution is found

## Acknowledgements

The Sudoku boards in `boards/mantere-koljonen` are
from

> Mantere, Timo and Janne Koljonen (2008). Solving and
> Analyzing Sudokus with Cultural Algorithms. In Proceedings
> of 2008 IEEE World Congress on Computational Intelligence
> (WCCI 2008), 1-6 June, Hong Kong, China, pages 4054-4061.

as made available
[here](http://lipas.uwasa.fi/~timan/sudoku/).
Thanks much to the authors for collecting these.

The [Al Escargot](https://www.sudokuwiki.org/Escargot) board
is by Arto Inkala.
