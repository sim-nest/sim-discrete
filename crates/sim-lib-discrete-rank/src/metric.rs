//! Nat-valued distance helpers for the discrete rank spaces.
//!
//! All metrics return a `BigUint` count or sum: Hamming, set symmetric
//! difference, Kendall-tau, and L1. Normalized fractional forms (e.g. Jaccard)
//! are the symmetric-difference numerator divided by the union size; the rank
//! layer uses the integer numerator as the coordinate distance.

use num_bigint::BigUint;
use std::collections::BTreeSet;

/// Hamming distance between two equal-length bit vectors.
pub fn hamming_bits(a: &[bool], b: &[bool]) -> BigUint {
    let count = a.iter().zip(b).filter(|(x, y)| x != y).count();
    BigUint::from(count)
}

/// Symmetric-difference cardinality of two sets of indices.
pub fn symmetric_difference(a: &[usize], b: &[usize]) -> BigUint {
    let sa: BTreeSet<usize> = a.iter().copied().collect();
    let sb: BTreeSet<usize> = b.iter().copied().collect();
    BigUint::from(sa.symmetric_difference(&sb).count())
}

/// Kendall-tau distance between two permutations of `0..n`: the number of pairs
/// ordered differently.
pub fn kendall_tau(a: &[usize], b: &[usize]) -> BigUint {
    let n = a.len();
    let mut count = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            let ord_a = a[i] < a[j];
            let ord_b = b[i] < b[j];
            if ord_a != ord_b {
                count += 1;
            }
        }
    }
    BigUint::from(count)
}

/// L1 distance between two equal-length integer vectors.
pub fn l1_u64(a: &[u64], b: &[u64]) -> BigUint {
    let mut acc = BigUint::from(0u32);
    for (x, y) in a.iter().zip(b) {
        acc += BigUint::from(x.abs_diff(*y));
    }
    acc
}

/// L1 distance between two equal-length `i64` coefficient vectors.
pub fn l1_i64(a: &[i64], b: &[i64]) -> BigUint {
    let mut acc = BigUint::from(0u32);
    for (x, y) in a.iter().zip(b) {
        let d = (*x as i128 - *y as i128).unsigned_abs();
        acc += BigUint::from(d);
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hamming_counts_differences() {
        assert_eq!(
            hamming_bits(&[true, false, true], &[true, true, false]),
            BigUint::from(2u32)
        );
    }

    #[test]
    fn symmetric_difference_counts() {
        assert_eq!(
            symmetric_difference(&[0, 1, 2], &[1, 2, 3]),
            BigUint::from(2u32)
        );
    }

    #[test]
    fn kendall_tau_basic() {
        // Reversing 3 elements gives 3 discordant pairs.
        assert_eq!(kendall_tau(&[0, 1, 2], &[2, 1, 0]), BigUint::from(3u32));
        assert_eq!(kendall_tau(&[0, 1, 2], &[0, 1, 2]), BigUint::from(0u32));
    }

    #[test]
    fn l1_distances() {
        assert_eq!(l1_u64(&[1, 5, 3], &[4, 1, 3]), BigUint::from(7u32));
        assert_eq!(l1_i64(&[-2, 3], &[1, -1]), BigUint::from(7u32));
    }
}
