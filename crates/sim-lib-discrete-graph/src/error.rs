//! Error type for discrete graph algorithms.

use sim_lib_discrete_algebra::AlgebraError;

/// Errors raised by graph construction, algorithms, and verifiers.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphError {
    /// The graph is disconnected, so the requested result does not exist.
    #[error("graph is disconnected")]
    Disconnected,
    /// A negative-weight cycle makes shortest paths undefined.
    #[error("graph has a negative-weight cycle")]
    NegativeCycle,
    /// A negative edge weight was supplied to an algorithm that forbids it.
    #[error("negative edge weight is not allowed here")]
    NegativeWeight,
    /// The algorithm was called on the wrong kind of graph.
    #[error("wrong graph kind: {0}")]
    WrongGraphKind(String),
    /// An edge referenced a node index outside the node range.
    #[error("invalid endpoint on edge {edge}: node {node} >= node count {len}")]
    InvalidEndpoint {
        /// The offending edge id.
        edge: usize,
        /// The out-of-range node index.
        node: usize,
        /// The number of nodes.
        len: usize,
    },
    /// A node index passed to an algorithm was outside the node range.
    #[error("node {node} out of range: node count {count}")]
    NodeOutOfRange {
        /// The out-of-range node index.
        node: usize,
        /// The number of nodes.
        count: usize,
    },
    /// A submitted certificate failed verification.
    #[error("certificate invalid: {0}")]
    CertificateInvalid(String),
    /// A staged feature is not yet implemented.
    #[error("unsupported: {0}")]
    Unsupported(String),
}

impl From<AlgebraError> for GraphError {
    fn from(err: AlgebraError) -> Self {
        match err {
            // For a min-plus adjacency matrix, a divergent closure star means a
            // negative-weight cycle.
            AlgebraError::NoStar => GraphError::NegativeCycle,
            other => GraphError::Unsupported(other.to_string()),
        }
    }
}
