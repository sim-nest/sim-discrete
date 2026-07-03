//! Exact combinatorial counting functions over `num_bigint::BigUint`.

use num_bigint::BigUint;

use crate::error::CombError;

/// Largest `n` accepted by [`factorial_checked`] before [`CombError::LimitExceeded`].
///
/// `factorial` folds an unbounded range of growing big-integer products; an
/// uncapped value parsed from untrusted text would hang or exhaust memory.
pub const MAX_FACTORIAL_INPUT: u64 = 50_000;

/// Largest `n` accepted by [`integer_partition_count_checked`] before
/// [`CombError::LimitExceeded`].
///
/// `integer_partition_count` allocates `vec![..; n + 1]` and runs an `O(n^2)`
/// big-integer recurrence, so an uncapped value is an out-of-memory / hang risk.
pub const MAX_PARTITION_INPUT: u64 = 10_000;

fn big(n: u64) -> BigUint {
    BigUint::from(n)
}

/// `n!` (with `factorial(0) == 1`).
pub fn factorial(n: u64) -> BigUint {
    (1..=n).fold(BigUint::from(1u32), |acc, i| acc * big(i))
}

/// `n!` with an explicit input ceiling ([`MAX_FACTORIAL_INPUT`]).
///
/// Returns [`CombError::LimitExceeded`] for `n` beyond the cap instead of
/// folding an unbounded range from untrusted input.
pub fn factorial_checked(n: u64) -> Result<BigUint, CombError> {
    if n > MAX_FACTORIAL_INPUT {
        return Err(CombError::LimitExceeded(format!(
            "factorial input {n} exceeds maximum {MAX_FACTORIAL_INPUT}"
        )));
    }
    Ok(factorial(n))
}

/// The falling factorial `n * (n-1) * ... * (n-k+1)` (`0` when `k > n`).
pub fn falling_factorial(n: u64, k: u64) -> BigUint {
    if k > n {
        return BigUint::from(0u32);
    }
    (0..k).fold(BigUint::from(1u32), |acc, i| acc * big(n - i))
}

/// The number of `k`-permutations of `n`, `nPk` (`0` when `k > n`).
pub fn permutation_count(n: u64, k: u64) -> BigUint {
    falling_factorial(n, k)
}

/// `n choose k` via the multiplicative formula (`0` when `k > n`).
///
/// # Examples
///
/// ```
/// use num_bigint::BigUint;
/// use sim_lib_discrete_comb::binomial;
///
/// assert_eq!(binomial(5, 2), BigUint::from(10u32));
/// assert_eq!(binomial(5, 0), BigUint::from(1u32)); // empty choice
/// assert_eq!(binomial(2, 5), BigUint::from(0u32)); // k > n
/// ```
pub fn binomial(n: u64, k: u64) -> BigUint {
    if k > n {
        return BigUint::from(0u32);
    }
    let k = k.min(n - k);
    let mut result = BigUint::from(1u32);
    for i in 1..=k {
        // result is always divisible by i at this step, so division is exact.
        result = result * big(n - k + i) / big(i);
    }
    result
}

/// The multinomial coefficient `(sum parts)! / prod(part!)`.
pub fn multinomial(parts: &[u64]) -> BigUint {
    let total: u64 = parts.iter().sum();
    let mut denom = BigUint::from(1u32);
    for &p in parts {
        denom *= factorial(p);
    }
    factorial(total) / denom
}

/// Stirling numbers of the second kind `S(n, k)`: partitions of an `n`-set into
/// `k` non-empty unlabeled blocks.
pub fn stirling2(n: u64, k: u64) -> BigUint {
    let (n, k) = (n as usize, k as usize);
    let mut dp = vec![BigUint::from(0u32); k + 1];
    dp[0] = BigUint::from(1u32); // S(0,0) = 1
    for _ in 1..=n {
        let mut next = vec![BigUint::from(0u32); k + 1];
        for j in 1..=k {
            next[j] = big(j as u64) * &dp[j] + &dp[j - 1];
        }
        dp = next;
    }
    if k < dp.len() {
        dp[k].clone()
    } else {
        BigUint::from(0u32)
    }
}

/// The Bell number `B(n) = sum_k S(n, k)`: total partitions of an `n`-set.
pub fn bell_number(n: u64) -> BigUint {
    (0..=n).map(|k| stirling2(n, k)).sum()
}

/// The partition count `p(n)`: ways to write `n` as a sum of positive integers,
/// order ignored.
pub fn integer_partition_count(n: u64) -> BigUint {
    let n = n as usize;
    let mut dp = vec![BigUint::from(0u32); n + 1];
    dp[0] = BigUint::from(1u32);
    for part in 1..=n {
        for j in part..=n {
            dp[j] = &dp[j] + &dp[j - part].clone();
        }
    }
    dp[n].clone()
}

/// `p(n)` with an explicit input ceiling ([`MAX_PARTITION_INPUT`]).
///
/// Returns [`CombError::LimitExceeded`] for `n` beyond the cap, so an untrusted
/// `n` cannot drive an unbounded allocation or `O(n^2)` recurrence.
pub fn integer_partition_count_checked(n: u64) -> Result<BigUint, CombError> {
    if n > MAX_PARTITION_INPUT {
        return Err(CombError::LimitExceeded(format!(
            "partition-count input {n} exceeds maximum {MAX_PARTITION_INPUT}"
        )));
    }
    Ok(integer_partition_count(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(n: u64) -> BigUint {
        BigUint::from(n)
    }

    #[test]
    fn factorial_basics() {
        assert_eq!(factorial(0), b(1));
        assert_eq!(factorial(5), b(120));
    }

    #[test]
    fn binomial_and_permutations() {
        assert_eq!(binomial(5, 2), b(10));
        assert_eq!(binomial(10, 5), b(252));
        assert_eq!(binomial(5, 7), b(0));
        assert_eq!(binomial(6, 0), b(1));
        assert_eq!(permutation_count(5, 3), b(60));
        assert_eq!(falling_factorial(5, 2), b(20));
    }

    #[test]
    fn multinomial_value() {
        // 4! / (2! 1! 1!) = 12
        assert_eq!(multinomial(&[2, 1, 1]), b(12));
    }

    #[test]
    fn stirling_bell_and_partitions() {
        assert_eq!(stirling2(4, 2), b(7));
        assert_eq!(bell_number(4), b(15));
        assert_eq!(integer_partition_count(5), b(7));
    }

    #[test]
    fn checked_counts_accept_small_inputs() {
        assert_eq!(factorial_checked(5).unwrap(), b(120));
        assert_eq!(integer_partition_count_checked(5).unwrap(), b(7));
    }

    #[test]
    fn checked_counts_reject_huge_inputs() {
        assert!(matches!(
            factorial_checked(MAX_FACTORIAL_INPUT + 1),
            Err(CombError::LimitExceeded(_))
        ));
        assert!(matches!(
            integer_partition_count_checked(u64::MAX),
            Err(CombError::LimitExceeded(_))
        ));
    }
}
