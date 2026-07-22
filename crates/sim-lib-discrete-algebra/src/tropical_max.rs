//! The bounded max-plus (tropical) semiring.
//!
//! Finite multiplication uses `i64::saturating_add`, so this is a bounded
//! tropical algebra for closure and matrix work.

use crate::semiring::Semiring;

/// Bounded max-plus tropical semiring: `add` is `max`, `mul` is saturating `+`.
///
/// `zero` is `NegInf`; `one` is `Fin(0)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxPlus {
    /// Negative infinity: the additive identity ("no path").
    NegInf,
    /// A finite tropical value.
    Fin(i64),
}

impl Semiring for MaxPlus {
    fn zero() -> Self {
        MaxPlus::NegInf
    }
    fn one() -> Self {
        MaxPlus::Fin(0)
    }
    fn add(&self, other: &Self) -> Self {
        match (self, other) {
            (MaxPlus::NegInf, x) | (x, MaxPlus::NegInf) => *x,
            (MaxPlus::Fin(a), MaxPlus::Fin(b)) => MaxPlus::Fin((*a).max(*b)),
        }
    }
    fn mul(&self, other: &Self) -> Self {
        match (self, other) {
            (MaxPlus::NegInf, _) | (_, MaxPlus::NegInf) => MaxPlus::NegInf,
            (MaxPlus::Fin(a), MaxPlus::Fin(b)) => MaxPlus::Fin(a.saturating_add(*b)),
        }
    }
    fn is_zero(&self) -> bool {
        matches!(self, MaxPlus::NegInf)
    }
    /// `1 + a + 2a + ... = max(0, a, 2a, ...)`. For `a <= 0` this is `0`; for a
    /// positive finite value the series diverges to `+inf`, so `None`.
    fn star(&self) -> Option<Self> {
        match self {
            MaxPlus::NegInf => Some(MaxPlus::Fin(0)),
            MaxPlus::Fin(a) if *a <= 0 => Some(MaxPlus::Fin(0)),
            MaxPlus::Fin(_) => None,
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
            MaxPlus::NegInf,
            MaxPlus::Fin(0),
            MaxPlus::Fin(-2),
            MaxPlus::Fin(-5),
        ]);
    }

    #[test]
    fn star_behaviour() {
        assert_eq!(MaxPlus::NegInf.star(), Some(MaxPlus::Fin(0)));
        assert_eq!(MaxPlus::Fin(0).star(), Some(MaxPlus::Fin(0)));
        assert_eq!(MaxPlus::Fin(-7).star(), Some(MaxPlus::Fin(0)));
        assert_eq!(MaxPlus::Fin(1).star(), None);
    }
}
