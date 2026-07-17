//! Fixed-width bit vectors in natural binary order, with rank/unrank.
//!
//! Position 0 is the least-significant bit, so ordinal `r` maps to the bit
//! vector whose bit `j` is `(r >> j) & 1`.

use crate::error::CombError;
use num_bigint::BigUint;

/// The largest width whose `2^width` cardinality fits a `u128` cursor.
const MAX_WIDTH: usize = 127;

/// Iterator over all `2^width` bit vectors of length `width`, in ordinal order.
#[derive(Debug, Clone)]
pub struct BitVectorIter {
    width: usize,
    next: u128,
    total: u128,
}

impl BitVectorIter {
    /// Total number of ordinals in this iterator's finite domain.
    pub fn total_ordinals(&self) -> u128 {
        self.total
    }

    /// Number of ordinals not yet emitted.
    pub fn remaining_ordinals(&self) -> u128 {
        self.total.saturating_sub(self.next)
    }
}

/// Construct a bit-vector iterator, rejecting widths that would overflow the
/// `u128` cursor.
pub fn bit_vectors(width: usize) -> Result<BitVectorIter, CombError> {
    if width > MAX_WIDTH {
        return Err(CombError::LimitExceeded(format!(
            "bit-vector width {width} exceeds {MAX_WIDTH}"
        )));
    }
    let total = 1u128 << width;
    Ok(BitVectorIter {
        width,
        next: 0,
        total,
    })
}

impl Iterator for BitVectorIter {
    type Item = Vec<bool>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.total {
            return None;
        }
        let r = self.next;
        self.next += 1;
        Some((0..self.width).map(|j| (r >> j) & 1 == 1).collect())
    }
}

/// The ordinal of a bit vector (position 0 = least significant bit).
pub fn bit_vector_rank(bits: &[bool]) -> BigUint {
    let mut rank = BigUint::from(0u32);
    for (j, &b) in bits.iter().enumerate() {
        if b {
            rank.set_bit(j as u64, true);
        }
    }
    rank
}

/// The bit vector of the given `width` for ordinal `rank`.
pub fn bit_vector_unrank(rank: &BigUint, width: usize) -> Result<Vec<bool>, CombError> {
    let bound = BigUint::from(1u32) << width;
    if rank >= &bound {
        return Err(CombError::OutOfRange {
            value: rank.to_string(),
            bound: bound.to_string(),
        });
    }
    Ok((0..width).map(|j| rank.bit(j as u64)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterates_all_in_order() {
        let all: Vec<_> = bit_vectors(2).unwrap().collect();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], vec![false, false]);
        assert_eq!(all[1], vec![true, false]); // ordinal 1 -> bit 0 set
        assert_eq!(all[3], vec![true, true]);
    }

    #[test]
    fn rank_unrank_round_trip() {
        for (i, bits) in bit_vectors(5).unwrap().enumerate() {
            let r = bit_vector_rank(&bits);
            assert_eq!(r, BigUint::from(i as u32));
            assert_eq!(bit_vector_unrank(&r, 5).unwrap(), bits);
        }
    }

    #[test]
    fn width_limit_enforced() {
        assert!(matches!(bit_vectors(200), Err(CombError::LimitExceeded(_))));
    }

    #[test]
    fn unrank_rejects_cardinality() {
        assert!(matches!(
            bit_vector_unrank(&BigUint::from(8u32), 3),
            Err(CombError::OutOfRange { .. })
        ));
    }

    #[test]
    fn width_127_total_is_exact_domain_size() {
        let iter = bit_vectors(127).unwrap();
        assert_eq!(iter.total_ordinals(), 1u128 << 127);
        assert_eq!(iter.remaining_ordinals(), 1u128 << 127);
    }
}
