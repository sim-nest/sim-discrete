# sim-discrete

Discrete math you can call from Rust: build and traverse graphs, count and rank
combinations, run semiring matrix closures, and take Walsh-Hadamard spectra --
each as a small library crate you add to your own project.

The SIM constellation ships a runtime and a `sim` CLI (`cargo install sim-run`;
full walkthrough in sim-say), but these crates are usable on their own.

## Examples

Build a directed graph and walk it breadth-first:

```rust
use sim_lib_discrete_graph::{bfs, Directedness, Graph};

let mut g: Graph<(), ()> = Graph::with_nodes(vec![(), (), ()], Directedness::Directed);
g.add_edge(0, 1, ()).unwrap();
g.add_edge(1, 2, ()).unwrap();

let t = bfs(&g, 0).unwrap();
assert_eq!(t.order, vec![0, 1, 2]);
assert_eq!(t.predecessor, vec![None, Some(0), Some(1)]);
```

```bash
cargo add sim-lib-discrete-graph
```

Count exactly, over arbitrary-precision integers -- "5 choose 2" is 10:

```rust
use num_bigint::BigUint;
use sim_lib_discrete_comb::binomial;

assert_eq!(binomial(5, 2), BigUint::from(10u32));
assert_eq!(binomial(2, 5), BigUint::from(0u32)); // k > n
```

```bash
cargo add sim-lib-discrete-comb num-bigint
```

Both snippets are the crates' own passing doctests
(`crates/sim-lib-discrete-graph/src/traversal.rs:35`,
`crates/sim-lib-discrete-comb/src/count.rs:67`).

## How it works

sim-discrete is the discrete-mathematics family of the SIM constellation. Where
the kernel supplies the `Value`/`Expr`/`Shape`/codec contracts and sim-numbers
supplies the number domains, tensors, and linear algebra, this repo supplies the
discrete-math behavior: a semiring spine with matrix-closure engines, graph
algorithms, exact combinatorics, Walsh-Hadamard spectral methods, and the rank
adapters that bind them together.

The spine is the organizing idea. Graph reachability and all-pairs shortest
paths are *derived* from semiring matrix closure rather than re-implemented per
algorithm, the combinatorial ordinals feed canonical rank-unrank, and the
spectral and algebraic invariants compile into rank grades. A thin facade crate
re-exports the sub-crates behind features and provides the kernel-free codecs,
the browse index, and the live `Lib`/`Cx` runtime registration.

## Crates

A facade plus a layered set of behavior crates. The algebra spine is the root
dependency; graph and spectral build on it, combinatorics stands alone, and rank
is the single crate permitted to depend on `sim-lib-rank`.

- `sim-lib-discrete` -- family facade that re-exports the sub-crates behind
  features and hosts the always-on `forms` (read-construct codecs) and `cards`
  (browse index) modules plus the `runtime`-gated kernel `Lib`/`Cx` op and claim
  registration.
- `sim-lib-discrete-algebra` -- the algebra spine: the `Semiring` trait and
  standard instances, plus a generic dense / coordinate-list sparse matrix with
  power and Kleene-closure engines from which reachability and shortest paths are
  derived.
- `sim-lib-discrete-comb` -- exact combinatorics: counting over `BigUint`, lazy
  enumerators, and canonical combinadic / Lehmer / mixed-radix rank-unrank
  helpers.
- `sim-lib-discrete-graph` -- graph value types, traversal, connectivity, MST,
  shortest paths, the graph <-> matrix bridge, and certificate-producing
  verifiers, with all-pairs paths and reachability as thin wrappers over the
  algebra spine's semiring closure.
- `sim-lib-discrete-spectral` -- the spectral atlas: the Fast Walsh-Hadamard
  Transform as the character transform of the boolean hypercube `(Z/2)^n`, with
  XOR convolution, subset zeta / Mobius transforms, Walsh signatures, and an
  implicit Hadamard matrix view.
- `sim-lib-discrete-rank` -- the only discrete crate allowed to depend on
  `sim-lib-rank`: finite rank spaces (bit-vector, subset, combination,
  permutation, simple-graph, FWHT-signal), rank metrics, and grade compilers that
  synthesize a rank grade from algebraic and spectral invariants.

### Rustdoc conventions

Public API documentation in `src/` follows one house style:

- Every public item opens with a one-line summary sentence, then context.
- The kernel defines the `Value`/`Expr`/`Shape`/codec contracts and sim-numbers
  supplies the number domains, tensors, and linear algebra; these crates supply
  the discrete-math behavior (algebra, combinatorics, graphs, spectral methods,
  ranking). Each item is framed by its discrete-math role.
- The first-reach types carry a `# Examples` doctest that compiles and passes.
- Cross-reference with intra-doc links, and link back to this README rather than
  restating it.

The public API is documentation-gated: each crate's `lib.rs` denies
`missing_docs`, so every public item, field, and variant must be documented for
the crate to build.

Each crate's runnable examples are its embedded `recipes/` tree plus the rustdoc
`# Examples` doctests; there are no stub recipe directories.

## Validation

The standalone repository gate is the same feature-aware surface used by CI and the PR checklist:

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo test --workspace --all-features
cargo run -p xtask -- simdoc --check
```

## Documentation Lanes

`cargo run -p xtask -- simdoc` builds the public documentation lanes:

- API docs: `target/doc/`
- Agent cards: `docs/agents/cards.jsonl` and `docs/agents/card-index.json`
- Human docs: `docs/humans/`
- Diagrams: `docs/diagrams/src/` and `docs/diagrams/generated/`

The same command writes split contract files under `docs/generated/`.
