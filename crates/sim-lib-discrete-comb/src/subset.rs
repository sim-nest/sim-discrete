//! Subsets of `{0, ..., n-1}` in bitmask order, with rank/unrank.
//!
//! A subset is represented as its sorted member indices. Ordinal `r` is the
//! bitmask whose bit `i` indicates membership of element `i`.

use crate::error::CombError;
use num_bigint::BigUint;

const MAX_N: usize = 127;

/// Iterator over all `2^n` subsets of `{0, ..., n-1}` in bitmask order.
#[derive(Debug, Clone)]
pub struct SubsetIter {
    n: usize,
    next: u128,
    total: u128,
}

/// Construct a subset iterator, rejecting `n` too large for the `u128` cursor.
pub fn subsets(n: usize) -> Result<SubsetIter, CombError> {
    if n > MAX_N {
        return Err(CombError::LimitExceeded(format!(
            "subset cardinality {n} exceeds {MAX_N}"
        )));
    }
    let total = if n == MAX_N { u128::MAX } else { 1u128 << n };
    Ok(SubsetIter { n, next: 0, total })
}

impl Iterator for SubsetIter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.total {
            return None;
        }
        let r = self.next;
        self.next += 1;
        Some((0..self.n).filter(|&i| (r >> i) & 1 == 1).collect())
    }
}

/// The bitmask ordinal of `subset` (a list of distinct indices `< n`).
pub fn subset_rank(subset: &[usize], n: usize) -> Result<BigUint, CombError> {
    let mut rank = BigUint::from(0u32);
    for &i in subset {
        if i >= n {
            return Err(CombError::OutOfRange {
                value: i.to_string(),
                bound: n.to_string(),
            });
        }
        rank.set_bit(i as u64, true);
    }
    Ok(rank)
}

/// The subset (sorted member indices) for bitmask ordinal `rank` over `n`.
pub fn subset_unrank(rank: &BigUint, n: usize) -> Vec<usize> {
    (0..n).filter(|&i| rank.bit(i as u64)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powerset_count_and_order() {
        let all: Vec<_> = subsets(3).unwrap().collect();
        assert_eq!(all.len(), 8);
        assert_eq!(all[0], Vec::<usize>::new());
        assert_eq!(all[1], vec![0]);
        assert_eq!(all[7], vec![0, 1, 2]);
    }

    #[test]
    fn rank_unrank_round_trip() {
        for (i, s) in subsets(4).unwrap().enumerate() {
            let r = subset_rank(&s, 4).unwrap();
            assert_eq!(r, BigUint::from(i as u32));
            assert_eq!(subset_unrank(&r, 4), s);
        }
    }

    #[test]
    fn rank_rejects_out_of_range() {
        assert!(matches!(
            subset_rank(&[5], 4),
            Err(CombError::OutOfRange { .. })
        ));
    }
}
