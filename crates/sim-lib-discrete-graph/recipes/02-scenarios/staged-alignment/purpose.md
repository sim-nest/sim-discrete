# Certified staged path and sequence alignment (descriptor)

Documents the reusable finite dynamic-programming owner used by music,
statistics, and analysis libraries. `layered_shortest_path` selects one state
from every non-empty layer and returns the complete Bellman values plus stable
backpointers. `dynamic_time_warp` aligns complete sequences or a query against a
subsequence with explicit gap, radius-window, boundary, and memory policies.
Full-memory mode returns a path and Bellman certificate; rolling mode retains
two score rows and returns a checkable final-row certificate without pretending
that a path was retained.

Both algorithms use `AlgorithmControl` for deterministic cell/edge charging,
memory bounds, deadlines, and cooperative cancellation. Integer overflow, NaN,
infinity, illegal windows, unreachable endpoints, and tampered certificates all
fail closed. Equal-cost transitions prefer the documented stable input order.

The discrete graph classes are not loaded by the cookbook sandbox, so this
recipe is a descriptor. Checked Rust fixtures in the crate exercise staged
backpointers, forbidden transitions, full edit paths, subsequence alignment,
rolling-memory evidence, bounded work, and certificate tampering.
