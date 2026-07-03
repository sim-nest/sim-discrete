//! The two-element field GF(2): `add` is XOR, `mul` is AND. Used for linear
//! algebra over `Z/2` (rank, coding theory).

use crate::semiring::Semiring;

/// The field GF(2): `add` is XOR, `mul` is AND.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gf2(pub bool);

impl Semiring for Gf2 {
    fn zero() -> Self {
        Gf2(false)
    }
    fn one() -> Self {
        Gf2(true)
    }
    fn add(&self, other: &Self) -> Self {
        Gf2(self.0 ^ other.0)
    }
    fn mul(&self, other: &Self) -> Self {
        Gf2(self.0 && other.0)
    }
    fn is_zero(&self) -> bool {
        !self.0
    }
    /// `1 + a + a^2 + ...` converges only for `a == 0` (giving `1`); for `a == 1`
    /// the XOR series `1 + 1 + 1 + ...` has no fixed point, so `None`.
    fn star(&self) -> Option<Self> {
        if self.0 { None } else { Some(Gf2(true)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semiring::laws::assert_semiring_laws;

    #[test]
    fn laws_hold() {
        assert_semiring_laws(&[Gf2(false), Gf2(true)]);
    }

    #[test]
    fn xor_add_and_star() {
        assert_eq!(Gf2(true).add(&Gf2(true)), Gf2(false), "1 XOR 1 == 0");
        assert_eq!(Gf2(false).star(), Some(Gf2(true)));
        assert_eq!(Gf2(true).star(), None);
    }
}
