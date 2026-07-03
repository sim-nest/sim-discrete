//! The real semiring over `f64` with ordinary `+` and `*`, plus a total order
//! for algorithms that need to compare entries. No `star` (left as default).

use crate::semiring::Semiring;

/// Real semiring: `add` is `+`, `mul` is `*`, `zero` is `0.0`, `one` is `1.0`.
///
/// Equality is the underlying `f64` equality (so `NaN` is never equal to
/// itself); algorithm fixtures should avoid `NaN`. [`RealF64::total_cmp`]
/// provides a total order for matrix algorithms that pivot or sort.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RealF64(pub f64);

impl RealF64 {
    /// Wrap a raw `f64`.
    pub fn new(value: f64) -> Self {
        RealF64(value)
    }
    /// A total order over all `f64` values (including `NaN` and signed zero),
    /// delegating to [`f64::total_cmp`].
    pub fn total_cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Semiring for RealF64 {
    fn zero() -> Self {
        RealF64(0.0)
    }
    fn one() -> Self {
        RealF64(1.0)
    }
    fn add(&self, other: &Self) -> Self {
        RealF64(self.0 + other.0)
    }
    fn mul(&self, other: &Self) -> Self {
        RealF64(self.0 * other.0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
    // star intentionally left as the default `None`: the geometric series
    // `1/(1-a)` is outside the scope of the structural closure engine.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semiring::laws::assert_semiring_laws;
    use core::cmp::Ordering;

    #[test]
    fn laws_hold() {
        // Small integers are exact in f64, so the semiring laws hold exactly.
        assert_semiring_laws(&[RealF64(0.0), RealF64(1.0), RealF64(2.0), RealF64(4.0)]);
    }

    #[test]
    fn total_order_is_total() {
        assert_eq!(RealF64(1.0).total_cmp(&RealF64(2.0)), Ordering::Less);
        assert_eq!(RealF64(2.0).total_cmp(&RealF64(2.0)), Ordering::Equal);
        assert_eq!(RealF64::zero().star(), None);
    }
}
