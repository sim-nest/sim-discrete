//! Finite additive costs shared by graph dynamic programs.

use core::{cmp::Ordering, fmt::Debug};

use crate::GraphError;

/// A finite, partially ordered cost with checked addition.
///
/// Floating-point implementations reject infinities and NaNs. Algorithms call
/// [`FiniteCost::compare`] rather than relying on an ambient total-order wrapper,
/// so non-finite values fail closed at the public boundary.
pub trait FiniteCost: Clone + Debug + PartialEq + PartialOrd {
    /// Additive identity.
    fn zero() -> Self;

    /// Exact or finite checked addition.
    fn checked_add(&self, rhs: &Self) -> Option<Self>;

    /// Whether this value is a valid finite algorithm input.
    fn is_finite(&self) -> bool {
        true
    }

    /// Compares two valid finite costs.
    fn compare(&self, rhs: &Self) -> Option<Ordering> {
        self.partial_cmp(rhs)
    }
}

macro_rules! integer_finite_cost {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl FiniteCost for $ty {
                fn zero() -> Self {
                    0
                }

                fn checked_add(&self, rhs: &Self) -> Option<Self> {
                    (*self).checked_add(*rhs)
                }
            }
        )+
    };
}

integer_finite_cost!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

macro_rules! float_finite_cost {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl FiniteCost for $ty {
                fn zero() -> Self {
                    0.0
                }

                fn checked_add(&self, rhs: &Self) -> Option<Self> {
                    let value = *self + *rhs;
                    value.is_finite().then_some(value)
                }

                fn is_finite(&self) -> bool {
                    <$ty>::is_finite(*self)
                }

                fn compare(&self, rhs: &Self) -> Option<Ordering> {
                    (self.is_finite() && rhs.is_finite()).then(|| self.total_cmp(rhs))
                }
            }
        )+
    };
}

float_finite_cost!(f32, f64);

pub(crate) fn add<C: FiniteCost>(left: &C, right: &C, context: &str) -> Result<C, GraphError> {
    left.checked_add(right)
        .filter(FiniteCost::is_finite)
        .ok_or_else(|| GraphError::WeightOverflow(context.to_owned()))
}

pub(crate) fn compare<C: FiniteCost>(
    left: &C,
    right: &C,
    context: &str,
) -> Result<Ordering, GraphError> {
    left.compare(right)
        .ok_or_else(|| GraphError::NonFiniteCost(context.to_owned()))
}

pub(crate) fn validate<C: FiniteCost>(cost: &C, context: &str) -> Result<(), GraphError> {
    if cost.is_finite() {
        Ok(())
    } else {
        Err(GraphError::NonFiniteCost(context.to_owned()))
    }
}
