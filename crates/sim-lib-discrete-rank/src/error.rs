//! Error type for the discrete rank adapters.

use sim_lib_discrete_comb::CombError;
use sim_lib_discrete_graph::GraphError;
use sim_lib_discrete_spectral::SpectralError;

/// Errors raised by discrete rank spaces, metrics, and grade compilers.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RankAdapterError {
    /// A staged rank adapter is not yet implemented.
    #[error("unsupported: {0}")]
    Unsupported(String),
    /// A rank / unrank round-trip invariant was violated.
    #[error("round-trip failure: {0}")]
    RoundTrip(String),
    /// An explicit enumeration or search limit was exceeded.
    #[error("limit exceeded: {0}")]
    LimitExceeded(String),
    /// A value was invalid for the space (wrong shape, out of range, etc.).
    #[error("invalid value: {0}")]
    Invalid(String),
}

impl From<CombError> for RankAdapterError {
    fn from(err: CombError) -> Self {
        RankAdapterError::Invalid(err.to_string())
    }
}

impl From<GraphError> for RankAdapterError {
    fn from(err: GraphError) -> Self {
        RankAdapterError::Invalid(err.to_string())
    }
}

impl From<SpectralError> for RankAdapterError {
    fn from(err: SpectralError) -> Self {
        RankAdapterError::Invalid(err.to_string())
    }
}
