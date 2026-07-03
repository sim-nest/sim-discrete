//! Finite-alphabet FWHT-signal rank space (coefficient L1 metric).

use crate::descriptor::{SpaceDescriptor, from_nat, to_nat};
use crate::error::RankAdapterError;
use crate::metric;
use num_bigint::BigUint;
use sim_lib_discrete_comb::{mixed_radix_rank, mixed_radix_unrank};
use sim_lib_rank::Nat;

/// `rank/discrete/fwht-signal`: integer signals of length `len` whose every
/// coefficient lies in the declared finite alphabet `0..alphabet`.
///
/// Rankable only because the alphabet is finite and declared; the ordinal is the
/// mixed-radix value of the coefficient vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FwhtSignalSpace {
    /// Signal length (number of coefficients).
    pub len: usize,
    /// Exclusive upper bound on each coefficient.
    pub alphabet: u64,
}

impl FwhtSignalSpace {
    fn radices(&self) -> Vec<u64> {
        vec![self.alphabet; self.len]
    }

    /// The space descriptor.
    pub fn descriptor(&self) -> SpaceDescriptor {
        SpaceDescriptor {
            id: "rank/discrete/fwht-signal",
            version: 1,
            params: vec![
                ("len", self.len.to_string()),
                ("alphabet", self.alphabet.to_string()),
            ],
            order: "mixed-radix",
            metric: "coefficient-l1",
        }
    }

    /// Total cardinality `alphabet^len`.
    pub fn cardinality(&self) -> Nat {
        to_nat(BigUint::from(self.alphabet).pow(self.len as u32))
    }

    /// Rank a coefficient vector (each value in `0..alphabet`).
    pub fn rank(&self, coeffs: &[i64]) -> Result<Nat, RankAdapterError> {
        if coeffs.len() != self.len {
            return Err(RankAdapterError::Invalid(format!(
                "signal length {} != {}",
                coeffs.len(),
                self.len
            )));
        }
        let mut digits = Vec::with_capacity(self.len);
        for &c in coeffs {
            if c < 0 || c as u64 >= self.alphabet {
                return Err(RankAdapterError::Invalid(format!(
                    "coefficient {c} outside alphabet 0..{}",
                    self.alphabet
                )));
            }
            digits.push(c as u64);
        }
        Ok(to_nat(mixed_radix_rank(&digits, &self.radices())?))
    }

    /// Unrank an ordinal into a coefficient vector.
    pub fn unrank(&self, ordinal: &Nat) -> Result<Vec<i64>, RankAdapterError> {
        let digits = mixed_radix_unrank(&from_nat(ordinal), &self.radices())?;
        Ok(digits.into_iter().map(|d| d as i64).collect())
    }

    /// Coefficient L1 distance between two ordinals.
    pub fn distance(&self, a: &Nat, b: &Nat) -> Result<Nat, RankAdapterError> {
        Ok(to_nat(metric::l1_i64(&self.unrank(a)?, &self.unrank(b)?)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nat(i: u32) -> Nat {
        to_nat(BigUint::from(i))
    }

    #[test]
    fn round_trips_and_cardinality() {
        let space = FwhtSignalSpace {
            len: 3,
            alphabet: 2,
        };
        assert_eq!(space.cardinality(), nat(8));
        for i in 0..8u32 {
            let s = space.unrank(&nat(i)).unwrap();
            assert_eq!(space.rank(&s).unwrap(), nat(i));
        }
    }

    #[test]
    fn coefficient_l1_distance() {
        let space = FwhtSignalSpace {
            len: 3,
            alphabet: 4,
        };
        let a = space.rank(&[3, 0, 1]).unwrap();
        let b = space.rank(&[0, 2, 1]).unwrap();
        assert_eq!(space.distance(&a, &b).unwrap(), nat(3 + 2));
    }

    #[test]
    fn out_of_alphabet_rejected() {
        let space = FwhtSignalSpace {
            len: 2,
            alphabet: 2,
        };
        assert!(matches!(
            space.rank(&[5, 0]),
            Err(RankAdapterError::Invalid(_))
        ));
    }
}
