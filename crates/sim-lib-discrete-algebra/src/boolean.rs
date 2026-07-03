//! The boolean semiring `(OR, AND)`. Closure gives transitive reachability.

use crate::semiring::Semiring;

/// The two-element boolean semiring: `add` is OR, `mul` is AND.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoolRing(pub bool);

impl Semiring for BoolRing {
    fn zero() -> Self {
        BoolRing(false)
    }
    fn one() -> Self {
        BoolRing(true)
    }
    fn add(&self, other: &Self) -> Self {
        BoolRing(self.0 || other.0)
    }
    fn mul(&self, other: &Self) -> Self {
        BoolRing(self.0 && other.0)
    }
    fn is_zero(&self) -> bool {
        !self.0
    }
    /// Reachability closure of a scalar is always reachable: `1 + a + ... = 1`.
    fn star(&self) -> Option<Self> {
        Some(BoolRing(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semiring::laws::assert_semiring_laws;

    #[test]
    fn laws_hold() {
        assert_semiring_laws(&[BoolRing(false), BoolRing(true)]);
    }

    #[test]
    fn star_is_always_true() {
        assert_eq!(BoolRing(false).star(), Some(BoolRing(true)));
        assert_eq!(BoolRing(true).star(), Some(BoolRing(true)));
    }
}
