//! The min-plus (tropical) semiring. Closure gives all-pairs shortest paths.

use crate::semiring::Semiring;

/// Min-plus tropical semiring: `add` is `min`, `mul` is saturating `+`.
///
/// `zero` is `Inf` (no path); `one` is `Fin(0)` (the empty path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinPlus {
    /// Positive infinity: the additive identity ("no path").
    Inf,
    /// A finite tropical value (for example, a path length).
    Fin(i64),
}

impl Semiring for MinPlus {
    fn zero() -> Self {
        MinPlus::Inf
    }
    fn one() -> Self {
        MinPlus::Fin(0)
    }
    fn add(&self, other: &Self) -> Self {
        match (self, other) {
            (MinPlus::Inf, x) | (x, MinPlus::Inf) => *x,
            (MinPlus::Fin(a), MinPlus::Fin(b)) => MinPlus::Fin((*a).min(*b)),
        }
    }
    fn mul(&self, other: &Self) -> Self {
        match (self, other) {
            (MinPlus::Inf, _) | (_, MinPlus::Inf) => MinPlus::Inf,
            (MinPlus::Fin(a), MinPlus::Fin(b)) => MinPlus::Fin(a.saturating_add(*b)),
        }
    }
    fn is_zero(&self) -> bool {
        matches!(self, MinPlus::Inf)
    }
    /// `1 + a + a^2 + ... = min(0, a, 2a, ...)`. For `a >= 0` this is `0`; for a
    /// negative finite value the series diverges to `-inf`, so `None`.
    fn star(&self) -> Option<Self> {
        match self {
            MinPlus::Inf => Some(MinPlus::Fin(0)),
            MinPlus::Fin(a) if *a >= 0 => Some(MinPlus::Fin(0)),
            MinPlus::Fin(_) => None,
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
            MinPlus::Inf,
            MinPlus::Fin(0),
            MinPlus::Fin(2),
            MinPlus::Fin(5),
        ]);
    }

    #[test]
    fn star_behaviour() {
        assert_eq!(MinPlus::Inf.star(), Some(MinPlus::Fin(0)));
        assert_eq!(MinPlus::Fin(0).star(), Some(MinPlus::Fin(0)));
        assert_eq!(MinPlus::Fin(7).star(), Some(MinPlus::Fin(0)));
        assert_eq!(MinPlus::Fin(-1).star(), None);
    }
}
