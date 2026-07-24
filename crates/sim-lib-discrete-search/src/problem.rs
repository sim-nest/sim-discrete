//! Problem and interruption traits.

/// Outcome of applying a choice or propagating a state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchStep<State> {
    /// Continue with the supplied state.
    Continue(State),
    /// Drop this prefix without treating the whole problem as impossible.
    Pruned {
        /// Deterministic reason recorded only in tests or diagnostics.
        reason: String,
    },
    /// Reject this state as locally infeasible.
    Infeasible {
        /// Deterministic reason recorded only in tests or diagnostics.
        reason: String,
    },
}

impl<State> SearchStep<State> {
    /// Build a pruned step with a stable reason.
    pub fn pruned(reason: impl Into<String>) -> Self {
        Self::Pruned {
            reason: reason.into(),
        }
    }

    /// Build an infeasible step with a stable reason.
    pub fn infeasible(reason: impl Into<String>) -> Self {
        Self::Infeasible {
            reason: reason.into(),
        }
    }
}

/// Interrupt source checked by the search loop between bounded operations.
pub trait SearchInterrupt {
    /// Return true when the caller wants the run to stop with a cancellation
    /// receipt.
    fn is_cancelled(&self) -> bool;
}

/// Interrupt source that never cancels.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeverInterrupt;

impl SearchInterrupt for NeverInterrupt {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Generic state-space problem consumed by [`crate::solve`].
pub trait SearchProblem {
    /// State carried by the frontier.
    type State: Clone;
    /// Deterministic choice type; choices are sorted before they are explored.
    type Choice: Clone + Ord;
    /// Finished output emitted by the search.
    type Output: Clone + std::fmt::Debug;

    /// Return the initial search state.
    fn initial_state(&self) -> Self::State;

    /// Append possible choices for `state` to `out`.
    fn expand(&self, state: &Self::State, out: &mut Vec<Self::Choice>);

    /// Apply one choice to a state, or prune that prefix.
    fn apply(&self, state: &Self::State, choice: &Self::Choice) -> SearchStep<Self::State>;

    /// Propagate generic CSP constraints after a state is produced.
    fn propagate(&self, state: Self::State) -> SearchStep<Self::State> {
        SearchStep::Continue(state)
    }

    /// Return a finished output if this state is terminal.
    fn finish(&self, state: &Self::State) -> Option<Self::Output>;

    /// Deterministic priority score for best-first and A-star search.
    fn score_state(&self, _state: &Self::State) -> i64 {
        0
    }

    /// Deterministic optimistic remaining estimate for A-star and beam search.
    fn estimate_remaining(&self, _state: &Self::State) -> i64 {
        0
    }

    /// Lower-bound score used by branch-and-bound minimization.
    fn bound(&self, _state: &Self::State) -> Option<i64> {
        None
    }

    /// Score for a finished output used by branch-and-bound minimization.
    fn output_score(&self, _output: &Self::Output) -> Option<i64> {
        None
    }
}
