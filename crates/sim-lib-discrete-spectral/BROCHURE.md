# sim-lib-discrete-spectral

In one line: break a pattern over on-or-off choices into its building blocks and read how it is really shaped.

## What it gives you

When your data is a pattern indexed by yes-or-no combinations -- which switches are on, which features are present -- this crate lets you look at it in a second, revealing way. It rewrites the pattern as a blend of simple underlying components, so you can see which broad tendencies dominate and which fine details barely register. From that view you can measure how concentrated or how spread out a pattern is, combine two patterns by their toggled differences, and roll information up or down across every subset of choices. It moves between the plain view and the component view quickly and exactly, so nothing is approximated away.

## Why you will be glad

- You can tell at a glance whether a pattern is dominated by a few strong tendencies or smeared across many weak ones.
- Combining and summarizing patterns over subsets becomes a direct operation instead of a hand-rolled loop.
- The transform is exact and reversible, so you can move to the component view and back without drift.

## Where it fits

This crate is the spectral, or component-analysis, corner of the discrete-math family in SIM. It builds on the shared algebra core and keeps every downstream user, such as music, entirely out of its own definitions. That separation lets it stay a clean, general instrument that any part of the system can point at a subset-shaped pattern to understand its structure.
