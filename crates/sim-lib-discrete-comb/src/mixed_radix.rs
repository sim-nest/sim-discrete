//! Bounded integer vectors under a mixed-radix numeral system, with rank/unrank.
//!
//! `digits[i]` ranges over `0..radices[i]`. Digit 0 is most significant.

use crate::error::CombError;
use num_bigint::BigUint;

fn check(digits: &[u64], radices: &[u64]) -> Result<(), CombError> {
    if digits.len() != radices.len() {
        return Err(CombError::InvalidParameters(
            "digits and radices length mismatch".to_string(),
        ));
    }
    for (i, (&d, &r)) in digits.iter().zip(radices).enumerate() {
        if r == 0 {
            return Err(CombError::InvalidParameters(format!("radix {i} is zero")));
        }
        if d >= r {
            return Err(CombError::OutOfRange {
                value: d.to_string(),
                bound: r.to_string(),
            });
        }
    }
    Ok(())
}

/// The ordinal of `digits` in the mixed-radix system `radices`.
pub fn mixed_radix_rank(digits: &[u64], radices: &[u64]) -> Result<BigUint, CombError> {
    check(digits, radices)?;
    let mut rank = BigUint::from(0u32);
    for (&d, &r) in digits.iter().zip(radices) {
        rank = rank * BigUint::from(r) + BigUint::from(d);
    }
    Ok(rank)
}

/// The digit vector for ordinal `rank` in the mixed-radix system `radices`.
pub fn mixed_radix_unrank(rank: &BigUint, radices: &[u64]) -> Result<Vec<u64>, CombError> {
    for (i, &r) in radices.iter().enumerate() {
        if r == 0 {
            return Err(CombError::InvalidParameters(format!("radix {i} is zero")));
        }
    }
    let mut remaining = rank.clone();
    let mut digits = vec![0u64; radices.len()];
    for i in (0..radices.len()).rev() {
        let r = BigUint::from(radices[i]);
        let d = &remaining % &r;
        remaining /= &r;
        digits[i] = d.iter_u64_digits().next().unwrap_or(0);
    }
    if remaining != BigUint::from(0u32) {
        let total: BigUint = radices.iter().map(|&r| BigUint::from(r)).product();
        return Err(CombError::OutOfRange {
            value: rank.to_string(),
            bound: total.to_string(),
        });
    }
    Ok(digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_is_horner() {
        // radices [3,2,4], digits [2,1,3] -> ((2*2 + 1)*4 + 3) = 23
        let r = mixed_radix_rank(&[2, 1, 3], &[3, 2, 4]).unwrap();
        assert_eq!(r, BigUint::from(23u32));
    }

    #[test]
    fn rank_unrank_round_trip() {
        let radices = [3u64, 2, 4];
        let total = 3 * 2 * 4;
        for i in 0..total {
            let digits = mixed_radix_unrank(&BigUint::from(i as u32), &radices).unwrap();
            assert_eq!(
                mixed_radix_rank(&digits, &radices).unwrap(),
                BigUint::from(i as u32)
            );
        }
    }

    #[test]
    fn out_of_range_digit_rejected() {
        assert!(matches!(
            mixed_radix_rank(&[3], &[3]),
            Err(CombError::OutOfRange { .. })
        ));
    }
}
