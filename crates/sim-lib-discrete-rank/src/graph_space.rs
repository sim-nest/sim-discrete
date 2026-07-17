//! Simple undirected graph rank space (adjacency upper-triangle bit order).

use crate::descriptor::{SpaceDescriptor, from_nat, to_nat};
use crate::error::RankAdapterError;
use num_bigint::BigUint;
use sim_lib_rank::Nat;
use std::collections::BTreeSet;

/// `rank/discrete/simple-graph`: labeled simple undirected graphs on `n` nodes,
/// ranked by the bitmask over the `C(n,2)` upper-triangle adjacency positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleGraphSpace {
    /// Number of nodes.
    pub n: usize,
    /// The canonical ordered list of `(i, j)` with `i < j`.
    pairs: Vec<(usize, usize)>,
}

impl SimpleGraphSpace {
    /// Construct the space for `n` nodes.
    pub fn new(n: usize) -> Self {
        let mut pairs = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                pairs.push((i, j));
            }
        }
        SimpleGraphSpace { n, pairs }
    }

    /// The space descriptor.
    pub fn descriptor(&self) -> SpaceDescriptor {
        SpaceDescriptor {
            id: "rank/discrete/simple-graph",
            version: 1,
            params: vec![("n", self.n.to_string())],
            order: "adjacency-upper-triangle",
            metric: "edge-symmetric-difference",
        }
    }

    /// Number of possible edges, `C(n, 2)`.
    pub fn edge_slots(&self) -> usize {
        self.pairs.len()
    }

    /// Total cardinality `2^C(n,2)`.
    pub fn cardinality(&self) -> Nat {
        to_nat(BigUint::from(1u32) << self.pairs.len())
    }

    /// Rank an edge set (each edge `(i, j)` with `i < j`).
    pub fn rank(&self, edges: &[(usize, usize)]) -> Result<Nat, RankAdapterError> {
        let mut rank = BigUint::from(0u32);
        let mut seen = BTreeSet::new();
        for &(i, j) in edges {
            let (lo, hi) = (i.min(j), i.max(j));
            if i == j || hi >= self.n {
                return Err(RankAdapterError::Invalid(format!(
                    "edge ({i},{j}) invalid for n={}",
                    self.n
                )));
            }
            if !seen.insert((lo, hi)) {
                return Err(RankAdapterError::Invalid(format!(
                    "duplicate edge ({lo},{hi}) for n={}",
                    self.n
                )));
            }
            let pos = self
                .pairs
                .iter()
                .position(|&p| p == (lo, hi))
                .expect("pair in range");
            rank.set_bit(pos as u64, true);
        }
        Ok(to_nat(rank))
    }

    /// Unrank an ordinal into a sorted edge set.
    pub fn unrank(&self, ordinal: &Nat) -> Result<Vec<(usize, usize)>, RankAdapterError> {
        let bits = from_nat(ordinal);
        let bound = BigUint::from(1u32) << self.pairs.len();
        if bits >= bound {
            return Err(RankAdapterError::Invalid(format!(
                "simple graph ordinal {bits} >= cardinality {bound}"
            )));
        }
        let mut edges = Vec::new();
        for (pos, &pair) in self.pairs.iter().enumerate() {
            if bits.bit(pos as u64) {
                edges.push(pair);
            }
        }
        Ok(edges)
    }

    /// Edge symmetric-difference distance between two ordinals.
    pub fn distance(&self, a: &Nat, b: &Nat) -> Result<Nat, RankAdapterError> {
        let ea: BTreeSet<_> = self.unrank(a)?.into_iter().collect();
        let eb: BTreeSet<_> = self.unrank(b)?.into_iter().collect();
        Ok(to_nat(BigUint::from(ea.symmetric_difference(&eb).count())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nat(i: u32) -> Nat {
        to_nat(BigUint::from(i))
    }

    #[test]
    fn round_trips_all_graphs_on_3_nodes() {
        let space = SimpleGraphSpace::new(3); // C(3,2) = 3 edge slots, 8 graphs.
        assert_eq!(space.edge_slots(), 3);
        assert_eq!(space.cardinality(), nat(8));
        for i in 0..8u32 {
            let edges = space.unrank(&nat(i)).unwrap();
            assert_eq!(space.rank(&edges).unwrap(), nat(i));
        }
    }

    #[test]
    fn edge_symmetric_difference() {
        let space = SimpleGraphSpace::new(4);
        let a = space.rank(&[(0, 1), (1, 2)]).unwrap();
        let b = space.rank(&[(1, 2), (2, 3)]).unwrap();
        // {01,12} vs {12,23}: symmetric difference {01,23} = 2.
        assert_eq!(space.distance(&a, &b).unwrap(), nat(2));
    }

    #[test]
    fn invalid_edge_rejected() {
        let space = SimpleGraphSpace::new(3);
        assert!(matches!(
            space.rank(&[(0, 9)]),
            Err(RankAdapterError::Invalid(_))
        ));
    }

    #[test]
    fn duplicate_edge_rejected() {
        let space = SimpleGraphSpace::new(3);
        assert!(matches!(
            space.rank(&[(0, 1), (1, 0)]),
            Err(RankAdapterError::Invalid(_))
        ));
    }

    #[test]
    fn unrank_rejects_cardinality() {
        let space = SimpleGraphSpace::new(3);
        assert!(matches!(
            space.unrank(&space.cardinality()),
            Err(RankAdapterError::Invalid(_))
        ));
    }
}
