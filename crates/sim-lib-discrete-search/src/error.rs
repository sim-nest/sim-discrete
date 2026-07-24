//! Error type for bounded search helpers.

use thiserror::Error;

/// Validation error returned by concrete search fixtures.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SearchError {
    /// The search control cannot be executed.
    #[error("invalid search control: {0}")]
    InvalidControl(String),
    /// The problem data cannot describe a finite deterministic search.
    #[error("invalid search problem: {0}")]
    InvalidProblem(String),
}
