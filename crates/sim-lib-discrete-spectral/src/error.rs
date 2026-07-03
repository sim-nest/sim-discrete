//! Error type for the discrete spectral atlas.

/// Errors raised by FWHT, convolution, and Walsh-domain operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpectralError {
    /// A transform was given input whose length is not a power of two.
    #[error("length {len} is not a power of two")]
    NonPowerOfTwoLength {
        /// The offending length.
        len: usize,
    },
    /// An integer inverse with `DivideByLength` hit a non-divisible coefficient.
    #[error("inverse not divisible by length {len}")]
    NonDivisibleInverse {
        /// The transform length the coefficients must divide by.
        len: usize,
    },
    /// A normalization mode is not valid for this element type.
    #[error("invalid normalization: {0}")]
    InvalidNormalization(String),
    /// A length computation would overflow.
    #[error("length overflow for input of size {len}")]
    LengthOverflow {
        /// The offending input length.
        len: usize,
    },
    /// Operand shapes are incompatible for the requested operation.
    #[error("shape mismatch: {0}")]
    ShapeMismatch(String),
    /// An explicit size limit was exceeded (for example, materializing a view).
    #[error("limit exceeded: {0}")]
    LimitExceeded(String),
    /// An exact integer transform overflowed `i64` and would lose precision.
    #[error("integer overflow in exact transform")]
    Overflow,
}
