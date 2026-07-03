//! Versioned descriptors and card content for discrete rank spaces.
//!
//! A descriptor's parameters are part of the space identity: two spaces with the
//! same `id` but different parameters are different spaces.

use num_bigint::BigUint;
use sim_lib_rank::Nat;

/// A versioned descriptor for a finite discrete rank space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceDescriptor {
    /// Stable rank-space symbol, e.g. `rank/discrete/subset`.
    pub id: &'static str,
    /// Descriptor version; bump on any identity-affecting change.
    pub version: u32,
    /// Identity-defining parameters as `(name, value)` pairs.
    pub params: Vec<(&'static str, String)>,
    /// The canonical order name.
    pub order: &'static str,
    /// The default metric name.
    pub metric: &'static str,
}

/// Wrap a `BigUint` as a rank `Nat`.
pub fn to_nat(value: BigUint) -> Nat {
    Nat::from_biguint(value)
}

/// Extract the `BigUint` from a rank `Nat`.
pub fn from_nat(value: &Nat) -> BigUint {
    value.as_biguint().clone()
}

/// A browse/help card descriptor for a discrete rank space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardSpec {
    /// Stable card key (the rank-space symbol).
    pub key: &'static str,
    /// One-line summary.
    pub summary: &'static str,
    /// The canonical order.
    pub order: &'static str,
    /// The default metric.
    pub metric: &'static str,
}
