# sim-lib-discrete-search

In one line: search a finite decision space with explicit budgets and a receipt
that says exactly what happened.

## What it gives you

This crate gives discrete algorithms a common way to explore choices without
silently running forever or hiding partial answers. A caller supplies a state,
ordered choices, pruning rules, propagation, and optional cost bounds. The
engine charges work for expansion, scoring, propagation, and output, then stops
deterministically when it completes, proves infeasibility, hits a bound, or is
cancelled.

## Why you will be glad

- Search order is explicit, so repeated runs emit results in the same order.
- Work, time, frontier, memory, and result bounds are visible in the receipt.
- Branch-and-bound, beam, A-star, prefix pruning, CSP propagation, and the
  two-stack adapter share one contract instead of becoming private loops.

## Where it fits

This crate is the bounded-search member of the discrete-math family. It stays
generic over problem states and outputs, so music, graph, ranking, and planning
code can reuse the same bounded exploration contract without depending on a
music-specific or graph-specific search loop.
