# sim-discrete

In one line: one front door to discrete math -- algebra, counting, graphs, ranking, and spectral analysis.

## What it gives you

A calculation core that answers many discrete-math questions by changing what add and multiply mean, tools to count and list every arrangement and give each its own number, network modeling that finds how pieces link and reach and cost, exact positions for discrete objects, and a way to break a pattern over on-or-off choices into its building blocks -- switch on only the parts you need.

## Why you will be glad

- You get one coherent family instead of a pile of unrelated math helpers.
- Exact counting, graph answers, matrix closure, ranking, and spectral views use shared assumptions, so results are easier to compare.
- Feature switches let small projects stay small while larger projects can bring in the full family.

## Where it fits

sim-discrete sits above the protocol kernel and beside the numeric crates. It supplies the discrete behavior: algebra, combinations, graphs, rank spaces, and Walsh-domain analysis. The facade crate gathers those focused crates for runtime use, while each underlying crate remains useful as a direct Rust dependency.
