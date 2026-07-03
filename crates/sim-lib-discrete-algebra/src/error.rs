//! Error type for the discrete algebra spine.

/// Errors raised by semiring construction and matrix operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AlgebraError {
    /// Operand shapes are incompatible for the requested operation.
    #[error("shape mismatch: {0}")]
    ShapeMismatch(String),
    /// A `from_rows` input had rows of unequal length.
    #[error("ragged matrix input: rows have unequal length")]
    Ragged,
    /// Closure is undefined: a diagonal entry's Kleene star does not converge
    /// (either the semiring defines no `star`, or the series diverges here, as
    /// with a negative cycle in min-plus or a directed cycle in counting).
    #[error("closure undefined: a diagonal entry has no convergent star")]
    NoStar,
    /// An explicit size or iteration limit was exceeded.
    #[error("limit exceeded: {0}")]
    LimitExceeded(String),
    /// An index was out of bounds.
    #[error("index out of bounds: index {index}, len {len}")]
    IndexOutOfBounds {
        /// The offending index.
        index: usize,
        /// The valid length.
        len: usize,
    },
}
