//! The [`Semiring`] trait: the algebraic core of the discrete-math family.
//!
//! A semiring is a set with two operations, `add` (combine alternatives) and
//! `mul` (chain in sequence), each with an identity (`zero` and `one`). One
//! generic matrix-closure engine over a semiring derives many graph algorithms:
//! boolean closure gives reachability, min-plus closure gives shortest paths,
//! counting powers count walks. See the instance modules for the standard
//! semirings.

/// A semiring: `(add, zero)` is a commutative monoid, `(mul, one)` is a monoid,
/// `mul` distributes over `add`, and `zero` annihilates under `mul`.
///
/// Implementations must uphold, for all `a`, `b`, `c`:
/// - `zero + a == a` and `a + zero == a`
/// - `a + b == b + a` and `(a + b) + c == a + (b + c)`
/// - `one * a == a` and `a * one == a`
/// - `(a * b) * c == a * (b * c)`
/// - `a * (b + c) == a*b + a*c` and `(a + b) * c == a*c + b*c`
/// - `zero * a == zero` and `a * zero == zero`
///
/// # Examples
///
/// The standard instances differ only in what `add` and `mul` mean. Over
/// [`MinPlus`](crate::MinPlus), `add` is `min` and `mul` is `+`, so chaining a
/// path costs the sum of its edges while combining alternatives keeps the
/// cheaper one:
///
/// ```
/// use sim_lib_discrete_algebra::{MinPlus, Semiring};
///
/// let zero = MinPlus::zero(); // Inf: "no path"
/// let one = MinPlus::one(); //  Fin(0): the empty path
/// assert_eq!(one, MinPlus::Fin(0));
///
/// // mul chains in sequence (sum of weights); add picks the cheaper route.
/// assert_eq!(MinPlus::Fin(2).mul(&MinPlus::Fin(3)), MinPlus::Fin(5));
/// assert_eq!(MinPlus::Fin(2).add(&MinPlus::Fin(3)), MinPlus::Fin(2));
///
/// // zero annihilates under mul and is the additive identity.
/// assert_eq!(zero.mul(&MinPlus::Fin(7)), zero);
/// assert_eq!(zero.add(&MinPlus::Fin(7)), MinPlus::Fin(7));
/// ```
pub trait Semiring: Clone + PartialEq + core::fmt::Debug {
    /// The additive identity, also meaning "no value" / "no path".
    fn zero() -> Self;
    /// The multiplicative identity.
    fn one() -> Self;
    /// Combine two alternatives.
    fn add(&self, other: &Self) -> Self;
    /// Chain two values in sequence.
    fn mul(&self, other: &Self) -> Self;
    /// Whether this value is the additive identity.
    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
    /// The Kleene star `1 + a + a^2 + ...` when it converges, else `None`.
    ///
    /// Present only for closed semirings. The default is `None`.
    fn star(&self) -> Option<Self> {
        None
    }
}

#[cfg(test)]
pub(crate) mod laws {
    use super::Semiring;

    /// Assert the semiring laws on a small sample set (all triples).
    ///
    /// `samples` should include `zero` and `one` plus a few ordinary values.
    /// For floating-point semirings, pick values that are exact in the format.
    pub(crate) fn assert_semiring_laws<S: Semiring>(samples: &[S]) {
        let zero = S::zero();
        let one = S::one();
        for a in samples {
            assert_eq!(zero.add(a), *a, "zero + a == a");
            assert_eq!(a.add(&zero), *a, "a + zero == a");
            assert_eq!(one.mul(a), *a, "one * a == a");
            assert_eq!(a.mul(&one), *a, "a * one == a");
            assert_eq!(zero.mul(a), zero, "zero * a == zero");
            assert_eq!(a.mul(&zero), zero, "a * zero == zero");
            assert_eq!(a.is_zero(), *a == zero, "is_zero agrees with == zero");
        }
        for a in samples {
            for b in samples {
                assert_eq!(a.add(b), b.add(a), "add is commutative");
                for c in samples {
                    assert_eq!(a.add(b).add(c), a.add(&b.add(c)), "add is associative");
                    assert_eq!(a.mul(b).mul(c), a.mul(&b.mul(c)), "mul is associative");
                    assert_eq!(
                        a.mul(&b.add(c)),
                        a.mul(b).add(&a.mul(c)),
                        "left distributive"
                    );
                    assert_eq!(
                        a.add(b).mul(c),
                        a.mul(c).add(&b.mul(c)),
                        "right distributive"
                    );
                }
            }
        }
    }
}
