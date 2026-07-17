//! Bit-vector and subset rank spaces (bitmask order).

use crate::descriptor::{SpaceDescriptor, from_nat, to_nat};
use crate::error::RankAdapterError;
use crate::limits::DiscreteRankLimits;
use crate::metric;
use num_bigint::BigUint;
use sim_lib_discrete_comb::{bit_vector_rank, bit_vector_unrank, subset_rank, subset_unrank};
use sim_lib_rank::Nat;

/// `rank/discrete/bit-vector`: fixed-width bit vectors in natural binary order.
///
/// # Examples
///
/// ```
/// use sim_lib_discrete_rank::BitVectorSpace;
/// use sim_lib_rank::Nat;
///
/// let space = BitVectorSpace { width: 4 };
/// let ordinal = Nat::from(5u64);
/// let bits = space.unrank(&ordinal).unwrap();
/// assert_eq!(space.rank(&bits).unwrap(), ordinal);
/// assert_eq!(space.cardinality(), Nat::from(16u64));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitVectorSpace {
    /// The vector width.
    pub width: usize,
}

impl BitVectorSpace {
    /// Build a bit-vector space after applying the default descriptor limits.
    pub fn try_new(width: usize) -> Result<Self, RankAdapterError> {
        DiscreteRankLimits::DEFAULT.check_bit_vector_width(width)?;
        Ok(Self { width })
    }

    fn validate(&self) -> Result<(), RankAdapterError> {
        DiscreteRankLimits::DEFAULT.check_bit_vector_width(self.width)
    }

    /// The space descriptor.
    pub fn descriptor(&self) -> SpaceDescriptor {
        SpaceDescriptor {
            id: "rank/discrete/bit-vector",
            version: 1,
            params: vec![("width", self.width.to_string())],
            order: "natural-binary",
            metric: "hamming",
        }
    }

    /// Total cardinality `2^width`.
    pub fn cardinality(&self) -> Nat {
        to_nat(BigUint::from(1u32) << self.width)
    }

    /// Rank a bit vector of length `width`.
    pub fn rank(&self, bits: &[bool]) -> Result<Nat, RankAdapterError> {
        self.validate()?;
        if bits.len() != self.width {
            return Err(RankAdapterError::Invalid(format!(
                "bit vector length {} != width {}",
                bits.len(),
                self.width
            )));
        }
        Ok(to_nat(bit_vector_rank(bits)))
    }

    /// Unrank an ordinal into a bit vector of length `width`.
    pub fn unrank(&self, ordinal: &Nat) -> Result<Vec<bool>, RankAdapterError> {
        self.validate()?;
        Ok(bit_vector_unrank(&from_nat(ordinal), self.width)?)
    }

    /// Hamming distance between two ordinals.
    pub fn distance(&self, a: &Nat, b: &Nat) -> Result<Nat, RankAdapterError> {
        Ok(to_nat(metric::hamming_bits(
            &self.unrank(a)?,
            &self.unrank(b)?,
        )))
    }
}

/// `rank/discrete/subset`: subsets of `{0, ..., n-1}` in bitmask order.
///
/// # Examples
///
/// ```
/// use sim_lib_discrete_rank::SubsetSpace;
/// use sim_lib_rank::Nat;
///
/// let space = SubsetSpace { n: 5 };
/// let ordinal = Nat::from(11u64);
/// let members = space.unrank(&ordinal).unwrap();
/// assert_eq!(space.rank(&members).unwrap(), ordinal);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubsetSpace {
    /// The ground-set size.
    pub n: usize,
}

impl SubsetSpace {
    /// Build a subset space after applying the default descriptor limits.
    pub fn try_new(n: usize) -> Result<Self, RankAdapterError> {
        DiscreteRankLimits::DEFAULT.check_subset_size(n)?;
        Ok(Self { n })
    }

    fn validate(&self) -> Result<(), RankAdapterError> {
        DiscreteRankLimits::DEFAULT.check_subset_size(self.n)
    }

    /// The space descriptor.
    pub fn descriptor(&self) -> SpaceDescriptor {
        SpaceDescriptor {
            id: "rank/discrete/subset",
            version: 1,
            params: vec![("n", self.n.to_string())],
            order: "bitmask",
            metric: "symmetric-difference",
        }
    }

    /// Total cardinality `2^n`.
    pub fn cardinality(&self) -> Nat {
        to_nat(BigUint::from(1u32) << self.n)
    }

    /// Rank a subset (sorted distinct member indices).
    pub fn rank(&self, members: &[usize]) -> Result<Nat, RankAdapterError> {
        self.validate()?;
        Ok(to_nat(subset_rank(members, self.n)?))
    }

    /// Unrank an ordinal into the subset member list.
    pub fn unrank(&self, ordinal: &Nat) -> Result<Vec<usize>, RankAdapterError> {
        self.validate()?;
        Ok(subset_unrank(&from_nat(ordinal), self.n)?)
    }

    /// Symmetric-difference distance between two ordinals.
    pub fn distance(&self, a: &Nat, b: &Nat) -> Result<Nat, RankAdapterError> {
        Ok(to_nat(metric::symmetric_difference(
            &self.unrank(a)?,
            &self.unrank(b)?,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_vector_round_trips_and_measures() {
        let space = BitVectorSpace { width: 4 };
        for i in 0..16u32 {
            let ord = to_nat(BigUint::from(i));
            let bits = space.unrank(&ord).unwrap();
            assert_eq!(space.rank(&bits).unwrap(), ord);
        }
        // Hamming(0b0011, 0b0101) = 2.
        let d = space
            .distance(&to_nat(BigUint::from(3u32)), &to_nat(BigUint::from(5u32)))
            .unwrap();
        assert_eq!(d, to_nat(BigUint::from(2u32)));
        assert_eq!(space.cardinality(), to_nat(BigUint::from(16u32)));
    }

    #[test]
    fn subset_round_trips() {
        let space = SubsetSpace { n: 5 };
        for i in 0..32u32 {
            let ord = to_nat(BigUint::from(i));
            let members = space.unrank(&ord).unwrap();
            assert_eq!(space.rank(&members).unwrap(), ord);
        }
    }

    #[test]
    fn subset_symmetric_difference() {
        let space = SubsetSpace { n: 4 };
        let a = space.rank(&[0, 1, 2]).unwrap();
        let b = space.rank(&[1, 2, 3]).unwrap();
        assert_eq!(space.distance(&a, &b).unwrap(), to_nat(BigUint::from(2u32)));
    }

    #[test]
    fn bit_vector_unrank_rejects_cardinality() {
        let space = BitVectorSpace { width: 4 };
        assert!(matches!(
            space.unrank(&space.cardinality()),
            Err(RankAdapterError::Invalid(_))
        ));
    }

    #[test]
    fn subset_rank_rejects_duplicates() {
        let space = SubsetSpace { n: 4 };
        assert!(matches!(
            space.rank(&[1, 1]),
            Err(RankAdapterError::Invalid(_))
        ));
    }

    #[test]
    fn subset_unrank_rejects_cardinality() {
        let space = SubsetSpace { n: 4 };
        assert!(matches!(
            space.unrank(&space.cardinality()),
            Err(RankAdapterError::Invalid(_))
        ));
    }

    #[test]
    fn checked_constructors_reject_first_out_of_range_dimensions() {
        assert!(BitVectorSpace::try_new(127).is_ok());
        assert!(matches!(
            BitVectorSpace::try_new(128),
            Err(RankAdapterError::LimitExceeded(_))
        ));
        assert!(SubsetSpace::try_new(127).is_ok());
        assert!(matches!(
            SubsetSpace::try_new(128),
            Err(RankAdapterError::LimitExceeded(_))
        ));
    }
}
