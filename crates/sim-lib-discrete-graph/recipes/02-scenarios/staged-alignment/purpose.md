# Certified staged path and sequence alignment

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

The runnable specimen composes both algorithms over generated finite data and
verifies both certificates before printing their stable states, path, score,
and work accounting. This is the reusable alignment seam for point-clustering,
music, and analysis callers: clustering can supply states or centroids, while
this owner supplies sequence alignment and staged optimization without a copied
statistics-side DP loop. Checked fixtures in the crate additionally exercise
forbidden transitions, subsequence alignment, rolling-memory evidence, bounded
work, non-finite refusal, and certificate tampering.
