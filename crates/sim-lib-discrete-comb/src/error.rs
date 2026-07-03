//! Error type for discrete combinatorics.

/// Errors raised by combinatorial counts, enumerators, and ordinals.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CombError {
    /// An explicit enumeration limit was exceeded.
    #[error("limit exceeded: {0}")]
    LimitExceeded(String),
    /// An ordinal or index fell outside the valid range of its family.
    #[error("out of range: {value} >= bound {bound}")]
    OutOfRange {
        /// The offending value.
        value: String,
        /// The exclusive upper bound.
        bound: String,
    },
    /// The supplied parameters do not describe a valid combinatorial family.
    #[error("invalid parameters: {0}")]
    InvalidParameters(String),
}
