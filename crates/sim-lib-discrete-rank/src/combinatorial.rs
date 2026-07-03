//! Combination, permutation, and bounded-integer-vector rank spaces.

use crate::descriptor::{SpaceDescriptor, from_nat, to_nat};
use crate::error::RankAdapterError;
use crate::metric;
use sim_lib_discrete_comb::{
    combination_rank, combination_unrank, mixed_radix_rank, mixed_radix_unrank, permutation_rank,
    permutation_unrank,
};
use sim_lib_rank::Nat;

/// `rank/discrete/combination`: `k`-subsets of `n` in combinadic order.
///
/// # Examples
///
/// ```
/// use sim_lib_discrete_rank::CombinationSpace;
/// use sim_lib_rank::Nat;
///
/// let space = CombinationSpace { n: 6, k: 3 };
/// let ordinal = Nat::from(7u64);
/// let combo = space.unrank(&ordinal).unwrap();
/// assert_eq!(space.rank(&combo).unwrap(), ordinal);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombinationSpace {
    /// Ground-set size.
    pub n: usize,
    /// Combination size.
    pub k: usize,
}

impl CombinationSpace {
    /// The space descriptor.
    pub fn descriptor(&self) -> SpaceDescriptor {
        SpaceDescriptor {
            id: "rank/discrete/combination",
            version: 1,
            params: vec![("n", self.n.to_string()), ("k", self.k.to_string())],
            order: "combinadic",
            metric: "symmetric-difference",
        }
    }

    /// Rank an ascending combination.
    pub fn rank(&self, combo: &[usize]) -> Result<Nat, RankAdapterError> {
        Ok(to_nat(combination_rank(combo, self.n)?))
    }

    /// Unrank an ordinal into a combination.
    pub fn unrank(&self, ordinal: &Nat) -> Result<Vec<usize>, RankAdapterError> {
        Ok(combination_unrank(&from_nat(ordinal), self.n, self.k)?)
    }

    /// Symmetric-difference distance between two ordinals.
    pub fn distance(&self, a: &Nat, b: &Nat) -> Result<Nat, RankAdapterError> {
        Ok(to_nat(metric::symmetric_difference(
            &self.unrank(a)?,
            &self.unrank(b)?,
        )))
    }
}

/// `rank/discrete/permutation`: permutations of `{0, ..., n-1}` in Lehmer order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermutationSpace {
    /// Permutation size.
    pub n: usize,
}

impl PermutationSpace {
    /// The space descriptor.
    pub fn descriptor(&self) -> SpaceDescriptor {
        SpaceDescriptor {
            id: "rank/discrete/permutation",
            version: 1,
            params: vec![("n", self.n.to_string())],
            order: "lehmer",
            metric: "kendall-tau",
        }
    }

    /// Rank a permutation.
    pub fn rank(&self, perm: &[usize]) -> Result<Nat, RankAdapterError> {
        Ok(to_nat(permutation_rank(perm)?))
    }

    /// Unrank an ordinal into a permutation.
    pub fn unrank(&self, ordinal: &Nat) -> Result<Vec<usize>, RankAdapterError> {
        Ok(permutation_unrank(&from_nat(ordinal), self.n)?)
    }

    /// Kendall-tau distance between two ordinals.
    pub fn distance(&self, a: &Nat, b: &Nat) -> Result<Nat, RankAdapterError> {
        Ok(to_nat(metric::kendall_tau(
            &self.unrank(a)?,
            &self.unrank(b)?,
        )))
    }
}

/// `rank/discrete/bounded-int-vector`: mixed-radix integer vectors (L1 metric).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedIntVectorSpace {
    /// Per-position radices; digit `i` ranges over `0..radices[i]`.
    pub radices: Vec<u64>,
}

impl BoundedIntVectorSpace {
    /// The space descriptor.
    pub fn descriptor(&self) -> SpaceDescriptor {
        SpaceDescriptor {
            id: "rank/discrete/bounded-int-vector",
            version: 1,
            params: vec![("radices", format!("{:?}", self.radices))],
            order: "mixed-radix",
            metric: "l1",
        }
    }

    /// Rank a digit vector.
    pub fn rank(&self, digits: &[u64]) -> Result<Nat, RankAdapterError> {
        Ok(to_nat(mixed_radix_rank(digits, &self.radices)?))
    }

    /// Unrank an ordinal into a digit vector.
    pub fn unrank(&self, ordinal: &Nat) -> Result<Vec<u64>, RankAdapterError> {
        Ok(mixed_radix_unrank(&from_nat(ordinal), &self.radices)?)
    }

    /// L1 distance between two ordinals.
    pub fn distance(&self, a: &Nat, b: &Nat) -> Result<Nat, RankAdapterError> {
        Ok(to_nat(metric::l1_u64(&self.unrank(a)?, &self.unrank(b)?)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    fn nat(i: u32) -> Nat {
        to_nat(BigUint::from(i))
    }

    #[test]
    fn combination_round_trips() {
        let space = CombinationSpace { n: 6, k: 3 };
        for i in 0..20u32 {
            let c = space.unrank(&nat(i)).unwrap();
            assert_eq!(space.rank(&c).unwrap(), nat(i));
        }
    }

    #[test]
    fn permutation_round_trips_and_kendall() {
        let space = PermutationSpace { n: 4 };
        for i in 0..24u32 {
            let p = space.unrank(&nat(i)).unwrap();
            assert_eq!(space.rank(&p).unwrap(), nat(i));
        }
        // [0,1,2,3] vs [3,2,1,0] is the maximal Kendall distance C(4,2)=6.
        let a = space.rank(&[0, 1, 2, 3]).unwrap();
        let b = space.rank(&[3, 2, 1, 0]).unwrap();
        assert_eq!(space.distance(&a, &b).unwrap(), nat(6));
    }

    #[test]
    fn bounded_int_vector_round_trips_and_l1() {
        let space = BoundedIntVectorSpace {
            radices: vec![3, 2, 4],
        };
        for i in 0..24u32 {
            let v = space.unrank(&nat(i)).unwrap();
            assert_eq!(space.rank(&v).unwrap(), nat(i));
        }
        let a = space.rank(&[2, 0, 3]).unwrap();
        let b = space.rank(&[0, 1, 1]).unwrap();
        assert_eq!(space.distance(&a, &b).unwrap(), nat(2 + 1 + 2));
    }
}
