//! `k`-combinations of `{0, ..., n-1}` in lexicographic order, with rank/unrank.
//!
//! A combination is a strictly ascending list of `k` indices. Rank and unrank
//! agree with the iterator's lexicographic order.

use crate::count::binomial;
use crate::error::CombError;
use num_bigint::BigUint;

/// Iterator over `k`-combinations of `{0, ..., n-1}` in lexicographic order.
#[derive(Debug, Clone)]
pub struct CombinationIter {
    n: usize,
    k: usize,
    current: Option<Vec<usize>>,
}

/// Construct a combination iterator. `k` must not exceed `n`.
///
/// # Examples
///
/// The `2`-combinations of `{0, 1, 2}` are produced in lexicographic order:
///
/// ```
/// use sim_lib_discrete_comb::combinations;
///
/// let all: Vec<Vec<usize>> = combinations(3, 2).unwrap().collect();
/// assert_eq!(all, vec![vec![0, 1], vec![0, 2], vec![1, 2]]);
///
/// // `k > n` is rejected.
/// assert!(combinations(2, 3).is_err());
/// ```
pub fn combinations(n: usize, k: usize) -> Result<CombinationIter, CombError> {
    if k > n {
        return Err(CombError::InvalidParameters(format!(
            "combinations: k={k} > n={n}"
        )));
    }
    Ok(CombinationIter {
        n,
        k,
        current: Some((0..k).collect()),
    })
}

impl Iterator for CombinationIter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.current.clone()?;
        // Advance to the next lexicographic combination.
        let mut next = cur.clone();
        let mut i = self.k;
        let advanced = loop {
            if i == 0 {
                break false;
            }
            i -= 1;
            if next[i] < self.n - self.k + i {
                next[i] += 1;
                for j in (i + 1)..self.k {
                    next[j] = next[j - 1] + 1;
                }
                break true;
            }
        };
        self.current = if advanced { Some(next) } else { None };
        Some(cur)
    }
}

fn validate(combo: &[usize], n: usize) -> Result<(), CombError> {
    for w in combo.windows(2) {
        if w[0] >= w[1] {
            return Err(CombError::InvalidParameters(
                "combination must be strictly ascending".to_string(),
            ));
        }
    }
    if let Some(&last) = combo.last()
        && last >= n
    {
        return Err(CombError::OutOfRange {
            value: last.to_string(),
            bound: n.to_string(),
        });
    }
    Ok(())
}

/// The lexicographic rank of `combo` among the `k`-combinations of `n`.
pub fn combination_rank(combo: &[usize], n: usize) -> Result<BigUint, CombError> {
    validate(combo, n)?;
    let k = combo.len();
    let mut rank = BigUint::from(0u32);
    let mut prev = 0usize;
    for (i, &c) in combo.iter().enumerate() {
        for v in prev..c {
            rank += binomial((n - 1 - v) as u64, (k - 1 - i) as u64);
        }
        prev = c + 1;
    }
    Ok(rank)
}

/// The `k`-combination of `n` at lexicographic ordinal `rank`.
///
/// Inverse of [`combination_rank`]: ranking then unranking returns the
/// original combination.
///
/// # Examples
///
/// ```
/// use num_bigint::BigUint;
/// use sim_lib_discrete_comb::{combination_rank, combination_unrank};
///
/// // Among the 2-combinations of {0,1,2}, [0,2] sits at ordinal 1.
/// assert_eq!(combination_unrank(&BigUint::from(1u32), 3, 2).unwrap(), vec![0, 2]);
///
/// // Round-trip rank/unrank.
/// let combo = vec![1usize, 3];
/// let r = combination_rank(&combo, 5).unwrap();
/// assert_eq!(combination_unrank(&r, 5, combo.len()).unwrap(), combo);
/// ```
pub fn combination_unrank(rank: &BigUint, n: usize, k: usize) -> Result<Vec<usize>, CombError> {
    let total = binomial(n as u64, k as u64);
    if rank >= &total {
        return Err(CombError::OutOfRange {
            value: rank.to_string(),
            bound: total.to_string(),
        });
    }
    let mut remaining = rank.clone();
    let mut combo = Vec::with_capacity(k);
    let mut v = 0usize;
    for i in 0..k {
        loop {
            let cnt = binomial((n - 1 - v) as u64, (k - 1 - i) as u64);
            if remaining < cnt {
                combo.push(v);
                v += 1;
                break;
            }
            remaining -= cnt;
            v += 1;
        }
    }
    Ok(combo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexicographic_order_and_count() {
        let all: Vec<_> = combinations(4, 2).unwrap().collect();
        assert_eq!(
            all,
            vec![
                vec![0, 1],
                vec![0, 2],
                vec![0, 3],
                vec![1, 2],
                vec![1, 3],
                vec![2, 3],
            ]
        );
        assert_eq!(combinations(5, 3).unwrap().count(), 10);
    }

    #[test]
    fn rank_unrank_round_trip() {
        for (i, c) in combinations(6, 3).unwrap().enumerate() {
            let r = combination_rank(&c, 6).unwrap();
            assert_eq!(r, BigUint::from(i as u32));
            assert_eq!(combination_unrank(&r, 6, 3).unwrap(), c);
        }
    }

    #[test]
    fn empty_combination_is_singleton() {
        assert_eq!(combinations(5, 0).unwrap().count(), 1);
    }
}
