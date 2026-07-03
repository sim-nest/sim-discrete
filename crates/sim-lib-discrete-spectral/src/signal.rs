//! Walsh-domain signal wrappers and normalization modes.

/// The ordering of Walsh functions in a transform's output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalshBasis {
    /// Natural (Hadamard) order, as produced by the recursive butterfly.
    Natural,
    /// Sequency order (sorted by number of sign changes).
    Sequency,
}

/// How an inverse transform is normalized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Normalization {
    /// No scaling (applying the raw transform twice scales by the length).
    None,
    /// Divide every coefficient by the transform length.
    DivideByLength,
    /// Scale by `1/sqrt(n)` (orthonormal); valid only for `f64`.
    OrthonormalF64,
}

/// A Walsh-domain signal: its coefficients and the basis they are expressed in.
#[derive(Debug, Clone, PartialEq)]
pub struct WalshSignal<T> {
    /// The coefficient values.
    pub values: Vec<T>,
    /// The basis ordering of `values`.
    pub basis: WalshBasis,
}

impl<T> WalshSignal<T> {
    /// Wrap `values` in the natural (Hadamard) basis.
    pub fn natural(values: Vec<T>) -> Self {
        WalshSignal {
            values,
            basis: WalshBasis::Natural,
        }
    }

    /// The number of coefficients.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the signal is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
