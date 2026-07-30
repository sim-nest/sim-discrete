# Certified minimum-cost assignment (descriptor)

Documents the generic assignment owner used by domain libraries. A dense cost
matrix is optimized together with explicit insertion, deletion, optional
doubling, and allow/forbid voice-crossing policy. The unrestricted polynomial
solver returns residual-flow dual potentials; the order-preserving polynomial
solver returns a Bellman suffix table. Both certificates are independently
checked, arithmetic fails closed on overflow, and equal optima use a documented
stable order.

The discrete domain classes are not loaded by the cookbook sandbox, so this
recipe is a descriptor. The checked Rust fixture in the crate exercises the
same contract, including tampered-certificate rejection.
