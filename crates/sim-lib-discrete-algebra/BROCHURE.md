# sim-lib-discrete-algebra

In one line: one calculation core that answers many discrete-math questions by changing what "add" and "multiply" mean.

## What it gives you

This is the shared spine the rest of the family stands on. It lets you treat a grid of numbers as a single object and ask deep questions about it: what can reach what, what is the cheapest way across, how many distinct paths exist. The trick is that all of these are the same computation with a different notion of combining values. Swap in plain counting and you count routes. Swap in "keep the smaller" and you get cheapest connections. Swap in true-or-false and you get pure reachability. Because one engine handles every case, results stay consistent and the same trusted machinery is reused instead of a fresh, separately checked routine for each question.

## Why you will be glad

- Reachability, cheapest-path, and path-counting answers all come from one well-tested core rather than scattered one-off code.
- You can plug in your own way of combining values and immediately reuse every closure and power calculation.
- Sparse and dense layouts are both handled, so small and large problems each stay efficient.

## Where it fits

This crate is the foundation layer of the discrete-math family in SIM. The graph tools and the spectral tools build directly on it instead of reinventing the same loops. It stays deliberately narrow: it knows only about combining values and the grids that hold them, and it depends on nothing from the wider system. That keeps it a stable base other pieces can lean on with confidence.
