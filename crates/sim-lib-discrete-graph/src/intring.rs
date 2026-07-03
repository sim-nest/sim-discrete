//! A plain integer ring as a [`Semiring`], for signed structural matrices.
//!
//! The algebra spine's tropical / boolean / counting semirings cannot represent
//! the signed entries of incidence and Laplacian matrices (no subtraction, no
//! negatives). `IntRing` is the ordinary ring of `i64` under `+` and `*`; it is
//! a valid semiring (every ring is), with no Kleene `star`.

use sim_lib_discrete_algebra::Semiring;

/// The ring of `i64` under saturating `+` and `*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntRing(pub i64);

impl Semiring for IntRing {
    fn zero() -> Self {
        IntRing(0)
    }
    fn one() -> Self {
        IntRing(1)
    }
    fn add(&self, other: &Self) -> Self {
        IntRing(self.0.saturating_add(other.0))
    }
    fn mul(&self, other: &Self) -> Self {
        IntRing(self.0.saturating_mul(other.0))
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_basics() {
        assert_eq!(IntRing(3).add(&IntRing(-5)), IntRing(-2));
        assert_eq!(IntRing(4).mul(&IntRing(-2)), IntRing(-8));
        assert!(IntRing::zero().is_zero());
        assert_eq!(IntRing(7).star(), None);
    }
}
