# sim-discrete

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

These commands run in the constellation workspace; only `sim-kernel` builds from a lone clone today (see `DEVELOPING.md` in `sim-sdk`). A single-repo build lands with the first crates.io publish.

```bash
cargo fmt --check && cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo doc --workspace --no-deps
cargo run -p xtask -- simdoc --check
```

## Documentation Lanes

`cargo run -p xtask -- simdoc` builds the public documentation lanes:

- API docs: `target/doc/`
- Agent cards: `docs/agents/cards.jsonl` and `docs/agents/card-index.json`
- Human docs: `docs/humans/`
- Diagrams: `docs/diagrams/src/` and `docs/diagrams/generated/`

The same command writes split contract files under `docs/generated/`.
