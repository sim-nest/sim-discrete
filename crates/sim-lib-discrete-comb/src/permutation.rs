//! Permutations of `{0, ..., n-1}` in lexicographic order, with Lehmer-code
//! rank/unrank.

use crate::count::factorial;
use crate::error::CombError;
use num_bigint::BigUint;

/// Iterator over permutations of `{0, ..., n-1}` in lexicographic order.
#[derive(Debug, Clone)]
pub struct PermutationIter {
    current: Option<Vec<usize>>,
}

/// Construct a permutation iterator over `{0, ..., n-1}`.
pub fn permutations(n: usize) -> PermutationIter {
    PermutationIter {
        current: Some((0..n).collect()),
    }
}

fn advance(p: &mut [usize]) -> bool {
    let n = p.len();
    if n < 2 {
        return false;
    }
    let mut i = n - 1;
    while i > 0 && p[i - 1] >= p[i] {
        i -= 1;
    }
    if i == 0 {
        return false;
    }
    let mut j = n - 1;
    while p[j] <= p[i - 1] {
        j -= 1;
    }
    p.swap(i - 1, j);
    p[i..].reverse();
    true
}

impl Iterator for PermutationIter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.current.clone()?;
        let mut next = cur.clone();
        self.current = if advance(&mut next) { Some(next) } else { None };
        Some(cur)
    }
}

fn to_usize(value: &BigUint) -> usize {
    value.iter_u64_digits().next().unwrap_or(0) as usize
}

fn validate(perm: &[usize]) -> Result<(), CombError> {
    let n = perm.len();
    let mut seen = vec![false; n];
    for &v in perm {
        if v >= n || seen[v] {
            return Err(CombError::InvalidParameters(
                "not a permutation of 0..n".to_string(),
            ));
        }
        seen[v] = true;
    }
    Ok(())
}

/// The Lehmer-code (lexicographic) rank of `perm`.
pub fn permutation_rank(perm: &[usize]) -> Result<BigUint, CombError> {
    validate(perm)?;
    let n = perm.len();
    let mut rank = BigUint::from(0u32);
    for i in 0..n {
        let smaller = (i + 1..n).filter(|&j| perm[j] < perm[i]).count();
        rank += BigUint::from(smaller as u64) * factorial((n - 1 - i) as u64);
    }
    Ok(rank)
}

/// The permutation of `{0, ..., n-1}` at lexicographic ordinal `rank`.
pub fn permutation_unrank(rank: &BigUint, n: usize) -> Result<Vec<usize>, CombError> {
    let total = factorial(n as u64);
    if rank >= &total {
        return Err(CombError::OutOfRange {
            value: rank.to_string(),
            bound: total.to_string(),
        });
    }
    let mut avail: Vec<usize> = (0..n).collect();
    let mut remaining = rank.clone();
    let mut perm = Vec::with_capacity(n);
    for i in 0..n {
        let f = factorial((n - 1 - i) as u64);
        let idx = to_usize(&(&remaining / &f));
        remaining %= &f;
        perm.push(avail.remove(idx));
    }
    Ok(perm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexicographic_order_and_count() {
        let all: Vec<_> = permutations(3).collect();
        assert_eq!(all.len(), 6);
        assert_eq!(all[0], vec![0, 1, 2]);
        assert_eq!(all[1], vec![0, 2, 1]);
        assert_eq!(all[5], vec![2, 1, 0]);
    }

    #[test]
    fn rank_unrank_round_trip() {
        for (i, p) in permutations(4).enumerate() {
            let r = permutation_rank(&p).unwrap();
            assert_eq!(r, BigUint::from(i as u32));
            assert_eq!(permutation_unrank(&r, 4).unwrap(), p);
        }
    }

    #[test]
    fn rank_rejects_non_permutation() {
        assert!(matches!(
            permutation_rank(&[0, 0, 1]),
            Err(CombError::InvalidParameters(_))
        ));
    }
}
