//! The counting semiring over `BigUint`. Powers count walks; closure counts
//! paths in a nilpotent (acyclic) setting.

use crate::semiring::Semiring;
use num_bigint::BigUint;

/// Natural-number counting semiring: `add` is `+`, `mul` is `*`.
///
/// `A^k[i][j]` over this semiring counts the walks of length `k` from `i` to
/// `j`. The closure converges only when the matrix is nilpotent (acyclic);
/// otherwise the closure engine raises a limit error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counting(pub BigUint);

impl Counting {
    /// Construct from a small machine integer for convenience.
    pub fn from_u64(value: u64) -> Self {
        Counting(BigUint::from(value))
    }
}

impl Semiring for Counting {
    fn zero() -> Self {
        Counting(BigUint::default())
    }
    fn one() -> Self {
        Counting(BigUint::from(1u32))
    }
    fn add(&self, other: &Self) -> Self {
        Counting(&self.0 + &other.0)
    }
    fn mul(&self, other: &Self) -> Self {
        Counting(&self.0 * &other.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == BigUint::default()
    }
    /// `1 + a + a^2 + ...` is finite only when `a == 0`, giving `1`.
    fn star(&self) -> Option<Self> {
        if self.is_zero() {
            Some(Self::one())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semiring::laws::assert_semiring_laws;

    #[test]
    fn laws_hold() {
        assert_semiring_laws(&[
            Counting::from_u64(0),
            Counting::from_u64(1),
            Counting::from_u64(3),
            Counting::from_u64(7),
        ]);
    }

    #[test]
    fn star_behaviour() {
        assert_eq!(Counting::from_u64(0).star(), Some(Counting::from_u64(1)));
        assert_eq!(Counting::from_u64(1).star(), None);
        assert_eq!(Counting::from_u64(9).star(), None);
    }
}
