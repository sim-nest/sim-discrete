//! Deterministic cookbook builders for bounded search recipes.

// conformance: Rust and codec/lisp-shaped bounded search specimens preserve result order and receipts.

use crate::{
    ConstrainedWordProblem, NeverInterrupt, SearchControl, SearchError, SearchOrder, SearchReceipt,
    WordSearchSolution, render_constrained_word_demo, solve,
};

/// Report produced by the constrained word cookbook recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstrainedWordDemo {
    /// Alphabet supplied by the caller.
    pub alphabet: Vec<String>,
    /// Target word length.
    pub length: usize,
    /// Required first symbol.
    pub required_first: String,
    /// Required last symbol.
    pub required_last: String,
    /// Maximum number of results requested by the caller.
    pub limit: usize,
    /// Rust-side solution order.
    pub rust_results: Vec<WordSearchSolution>,
    /// Lisp-shaped solution order.
    pub lisp_results: Vec<WordSearchSolution>,
    /// Rust-side receipt.
    pub rust_receipt: SearchReceipt,
    /// Lisp-shaped receipt.
    pub lisp_receipt: SearchReceipt,
    /// Stable Rust rendering.
    pub rust_rendered: String,
    /// Stable Lisp-shaped rendering.
    pub lisp_rendered: String,
}

/// Build the constrained word search report used by the cookbook.
pub fn constrained_word_demo(
    alphabet: Vec<String>,
    length: usize,
    required_first: String,
    required_last: String,
    limit: usize,
) -> Result<ConstrainedWordDemo, SearchError> {
    let problem = ConstrainedWordProblem::new(
        alphabet.clone(),
        length,
        required_first.clone(),
        required_last.clone(),
    )?;
    let control = SearchControl::default()
        .with_order(SearchOrder::AStar)
        .with_seed(13)
        .with_max_work(4_096)
        .with_max_results(limit)
        .with_max_frontier(128)
        .with_max_memory_nodes(256);

    let rust = solve(&problem, control.clone(), &NeverInterrupt);
    let lisp = solve(&problem, control, &NeverInterrupt);
    let rust_rendered = render_constrained_word_demo(&rust.outputs, &rust.receipt.digest);
    let lisp_rendered = render_constrained_word_demo(&lisp.outputs, &lisp.receipt.digest);

    Ok(ConstrainedWordDemo {
        alphabet,
        length,
        required_first,
        required_last,
        limit,
        rust_results: rust.outputs,
        lisp_results: lisp.outputs,
        rust_receipt: rust.receipt,
        lisp_receipt: lisp.receipt,
        rust_rendered,
        lisp_rendered,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SearchInterrupt, SearchProblem, SearchStatus, SearchStep, TwoStackAdapter, WorkCosts,
    };
    use std::{cell::Cell, time::Duration};

    fn fixture_problem() -> ConstrainedWordProblem {
        ConstrainedWordProblem::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            4,
            "A".to_string(),
            "C".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn rust_and_lisp_word_specimens_are_byte_identical() {
        let demo = constrained_word_demo(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            4,
            "A".to_string(),
            "C".to_string(),
            8,
        )
        .unwrap();
        assert_eq!(demo.rust_results, demo.lisp_results);
        assert_eq!(demo.rust_receipt, demo.lisp_receipt);
        assert_eq!(demo.rust_rendered, demo.lisp_rendered);
        assert_eq!(demo.rust_receipt.status, SearchStatus::Complete);
        assert_eq!(
            demo.rust_results
                .iter()
                .map(|solution| solution.word.join(""))
                .collect::<Vec<_>>(),
            vec!["ACAC", "ABAC"]
        );
    }

    #[test]
    fn result_bound_returns_partial_receipt() {
        let run = solve(
            &fixture_problem(),
            SearchControl::default()
                .with_order(SearchOrder::AStar)
                .with_max_results(1)
                .with_max_work(1_000),
            &NeverInterrupt,
        );
        assert_eq!(run.receipt.status, SearchStatus::Partial);
        assert_eq!(run.receipt.reason.as_deref(), Some("result bound reached"));
        assert_eq!(run.outputs.len(), 1);
    }

    #[test]
    fn zero_result_bound_emits_no_outputs() {
        let run = solve(
            &fixture_problem(),
            SearchControl::default()
                .with_order(SearchOrder::AStar)
                .with_max_results(0)
                .with_max_work(1_000),
            &NeverInterrupt,
        );
        assert_eq!(run.receipt.status, SearchStatus::Partial);
        assert_eq!(run.receipt.reason.as_deref(), Some("result bound reached"));
        assert!(run.outputs.is_empty());
    }

    #[test]
    fn work_and_time_bounds_return_partial_receipts() {
        let work = solve(
            &fixture_problem(),
            SearchControl::default()
                .with_order(SearchOrder::BreadthFirst)
                .with_max_work(1),
            &NeverInterrupt,
        );
        assert_eq!(work.receipt.status, SearchStatus::Partial);
        assert_eq!(work.receipt.reason.as_deref(), Some("work bound reached"));

        let time = solve(
            &fixture_problem(),
            SearchControl::default().with_max_time(Duration::ZERO),
            &NeverInterrupt,
        );
        assert_eq!(time.receipt.status, SearchStatus::Partial);
        assert_eq!(time.receipt.reason.as_deref(), Some("time bound reached"));
    }

    #[test]
    fn frontier_and_memory_bounds_are_enforced() {
        let frontier = solve(
            &fixture_problem(),
            SearchControl::default()
                .with_order(SearchOrder::BreadthFirst)
                .with_max_frontier(0)
                .with_max_work(1_000),
            &NeverInterrupt,
        );
        assert_eq!(frontier.receipt.status, SearchStatus::Partial);
        assert_eq!(
            frontier.receipt.reason.as_deref(),
            Some("frontier bound reached")
        );

        let memory = solve(
            &fixture_problem(),
            SearchControl::default()
                .with_order(SearchOrder::BreadthFirst)
                .with_max_memory_nodes(0)
                .with_max_work(1_000),
            &NeverInterrupt,
        );
        assert_eq!(memory.receipt.status, SearchStatus::Partial);
        assert_eq!(
            memory.receipt.reason.as_deref(),
            Some("memory node bound reached")
        );
    }

    #[test]
    fn cancellation_returns_cancelled_receipt() {
        struct CancelImmediately(Cell<bool>);
        impl SearchInterrupt for CancelImmediately {
            fn is_cancelled(&self) -> bool {
                self.0.replace(true)
            }
        }

        let interrupt = CancelImmediately(Cell::new(true));
        let run = solve(
            &fixture_problem(),
            SearchControl::default().with_max_work(1_000),
            &interrupt,
        );
        assert_eq!(run.receipt.status, SearchStatus::Cancelled);
        assert!(run.outputs.is_empty());
    }

    #[test]
    fn infeasible_problem_returns_infeasible_receipt() {
        let problem = ConstrainedWordProblem::new(
            vec!["A".to_string(), "B".to_string()],
            2,
            "A".to_string(),
            "A".to_string(),
        )
        .unwrap();
        let run = solve(
            &problem,
            SearchControl::default()
                .with_order(SearchOrder::BreadthFirst)
                .with_max_work(1_000),
            &NeverInterrupt,
        );
        assert_eq!(run.receipt.status, SearchStatus::Infeasible);
        assert!(run.outputs.is_empty());
    }

    #[test]
    fn branch_and_bound_prunes_after_best_score() {
        let run = solve(
            &fixture_problem(),
            SearchControl::default()
                .with_order(SearchOrder::AStar)
                .with_branch_and_bound(true)
                .with_max_work(1_000),
            &NeverInterrupt,
        );
        assert_eq!(run.receipt.status, SearchStatus::Complete);
        assert_eq!(
            run.outputs
                .iter()
                .map(|solution| solution.word.join(""))
                .collect::<Vec<_>>(),
            vec!["ACAC"]
        );
        assert!(run.receipt.pruned > 0);
    }

    #[test]
    fn beam_search_keeps_width_bound() {
        let run = solve(
            &fixture_problem(),
            SearchControl::default()
                .with_order(SearchOrder::Beam { width: 1 })
                .with_max_work(1_000),
            &NeverInterrupt,
        );
        assert_eq!(run.receipt.status, SearchStatus::Complete);
        assert_eq!(run.receipt.max_frontier, 1);
        assert_eq!(run.outputs[0].word.join(""), "ACAC");
    }

    #[test]
    fn charges_each_work_class_and_records_policy_digest() {
        let costs = WorkCosts {
            expand: 2,
            score: 3,
            propagate: 5,
            emit: 7,
        };
        let run = solve(
            &fixture_problem(),
            SearchControl::default()
                .with_order(SearchOrder::BestFirst)
                .with_seed(99)
                .with_costs(costs)
                .with_max_work(2_000),
            &NeverInterrupt,
        );
        let expected = run.receipt.expanded * costs.expand
            + run.receipt.scored * costs.score
            + run.receipt.propagated * costs.propagate
            + run.receipt.emitted * costs.emit;
        assert_eq!(run.receipt.work_used, expected);

        let other = solve(
            &fixture_problem(),
            SearchControl::default()
                .with_order(SearchOrder::BestFirst)
                .with_seed(100)
                .with_costs(costs)
                .with_max_work(2_000),
            &NeverInterrupt,
        );
        assert_ne!(run.receipt.policy_digest, other.receipt.policy_digest);
    }

    #[test]
    fn two_stack_adapter_supports_layered_backtracking() {
        let mut frontier = TwoStackAdapter::with_root(vec!["A"]);
        assert_eq!(frontier.pop(), Some(vec!["A"]));
        frontier.push_next(vec!["A", "B"]);
        frontier.push_next(vec!["A", "C"]);
        assert_eq!(frontier.pop(), Some(vec!["A", "B"]));
        assert_eq!(frontier.pop(), Some(vec!["A", "C"]));
        assert_eq!(frontier.depth(), 1);
    }

    #[derive(Clone)]
    struct PropagatingProblem;

    impl SearchProblem for PropagatingProblem {
        type State = usize;
        type Choice = usize;
        type Output = usize;

        fn initial_state(&self) -> Self::State {
            0
        }

        fn expand(&self, _state: &Self::State, out: &mut Vec<Self::Choice>) {
            out.extend([1, 2]);
        }

        fn apply(&self, state: &Self::State, choice: &Self::Choice) -> SearchStep<Self::State> {
            SearchStep::Continue(state + choice)
        }

        fn propagate(&self, state: Self::State) -> SearchStep<Self::State> {
            if state == 2 {
                SearchStep::pruned("generic CSP propagation rejected value")
            } else {
                SearchStep::Continue(state)
            }
        }

        fn finish(&self, state: &Self::State) -> Option<Self::Output> {
            (*state >= 3).then_some(*state)
        }
    }

    #[test]
    fn generic_csp_propagation_prunes_child_states() {
        let run = solve(
            &PropagatingProblem,
            SearchControl::default()
                .with_order(SearchOrder::BreadthFirst)
                .with_max_results(1)
                .with_max_work(1_000),
            &NeverInterrupt,
        );
        assert_eq!(run.outputs, vec![3]);
        assert!(run.receipt.pruned > 0);
    }
}
