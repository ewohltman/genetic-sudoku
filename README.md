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
          Population per generation [default: 75]
  -s, --selection-rate <SELECTION_RATE>
          Fraction of population selected [default: 0.5]
  -m, --mutation-rate <MUTATION_RATE>
          Mutation rate as fraction [default: 0.06]
  -e, --elitism <ELITISM>
          Number of top individuals carried unchanged into the next generation (0 = disabled) [default: 0]
  -l, --local-search <LOCAL_SEARCH>
          Greedy local-search passes applied to each child (0 = disabled) [default: 0]
  -r, --restart <RESTART>
          Number of generations without improvement before restarting with a new random population (0 = disabled) [default: 0]
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

The optional `--elitism` argument specifies how many of the fittest individuals
are carried unchanged into the next generation, guaranteeing the best solution
found so far is never lost to crossover or mutation. It is disabled by default.
On its own it can accelerate premature convergence on easy boards, so it is most
useful in combination with `--local-search` and `--restart` (see below).

The optional `--local-search` argument enables a memetic (hybrid) refinement
step: after each child is bred, the given number of greedy hill-climb passes set
every non-fixed cell to the digit that minimizes its local conflicts. This helps
the algorithm make the final descent to a solution on hard puzzles, at the cost
of fewer generations per second. It is disabled by default.

The optional `--restart` argument specifies the number of generations without
an improvement in the best fitness score before the population is discarded
and regenerated from scratch. This can help escape local optima on harder
puzzles. It is disabled by default.

Together, these three arguments make the historically hard *Al Escargot* puzzle
solvable in a couple of seconds, where the base genetic algorithm never solves
it:

```bash
genetic-sudoku -p 1000 -s 0.4 -m 0.05 --elitism 2 --local-search 2 --restart 50 boards/al-escargot.txt
```

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
    * If `--local-search` is enabled, greedily refine each child by setting its
      non-fixed cells to the digits that minimize their local conflicts
  * Carry the fittest `--elitism` individuals into the next generation unchanged
    so the best solution found is never lost
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

### References

The elitism and local-search (memetic) enhancements were motivated by the
following research on genetic algorithms for Sudoku:

* Mantere, Timo and Janne Koljonen (2007). *Solving, rating and generating
  Sudoku puzzles with GA.* IEEE Congress on Evolutionary Computation (CEC 2007),
  pages 1382-1389.
  <https://www.researchgate.net/publication/224301656_Solving_rating_and_generating_Sudoku_puzzles_with_GA>
* Deng, Xiu Qin and Yong Da Li (2013). *A novel hybrid genetic algorithm for
  solving Sudoku puzzles.* Optimization Letters, 7(2), pages 241-257.
  <https://www.math.uci.edu/~brusso/DengLiOptLett2013.pdf>
* Sato, Yuji and Hazuki Inoue (2010). *Solving Sudoku with genetic operations
  that preserve building blocks.* IEEE Conference on Computational Intelligence
  and Games (CIG 2010). <https://ieeexplore.ieee.org/document/5593375/>
* Segura, Carlos et al. (2018). *Solving Sudoku's by Evolutionary Algorithms
  with Pre-processing.* In Studies in Computational Intelligence.
  <https://link.springer.com/chapter/10.1007/978-3-319-97888-8_1>
