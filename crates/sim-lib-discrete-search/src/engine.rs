//! Deterministic bounded search engine.

use std::{cmp::Ordering, time::Instant};

use crate::{
    SearchControl, SearchInterrupt, SearchOrder, SearchProblem, SearchReceipt, SearchRun,
    SearchStatus, SearchStep, receipt::Metrics,
};

#[derive(Clone, Debug)]
struct Node<State> {
    state: State,
    sequence: u64,
    priority: i64,
}

#[derive(Clone, Debug)]
struct Stop {
    status: SearchStatus,
    reason: String,
}

/// Solve a search problem under explicit deterministic bounds.
pub fn solve<P>(
    problem: &P,
    control: SearchControl,
    interrupt: &dyn SearchInterrupt,
) -> SearchRun<P::Output>
where
    P: SearchProblem,
{
    if let Err(reason) = control.validate() {
        return SearchRun {
            outputs: Vec::new(),
            receipt: SearchReceipt::invalid(&control, reason),
        };
    }

    let started = Instant::now();
    let mut metrics = Metrics::default();
    let mut outputs = Vec::new();
    let mut next_sequence = 0u64;
    let mut best_score = None;

    let initial = match propagate(problem, problem.initial_state(), &control, &mut metrics) {
        Ok(Some(state)) => state,
        Ok(None) => {
            return finish(control, outputs, metrics, SearchStatus::Infeasible, None);
        }
        Err(stop) => return finish(control, outputs, metrics, stop.status, Some(stop.reason)),
    };
    let initial_priority = priority(problem, &initial, 0, control.order);
    let mut frontier = vec![Node {
        state: initial,
        sequence: next_sequence,
        priority: initial_priority,
    }];
    next_sequence += 1;
    update_frontier(&mut metrics, frontier.len());
    if let Err(stop) = enforce_collection_limits(&frontier, &outputs, &control, &metrics) {
        return finish(control, outputs, metrics, stop.status, Some(stop.reason));
    }

    let stop = loop {
        if interrupt.is_cancelled() {
            break Some(Stop {
                status: SearchStatus::Cancelled,
                reason: "interrupt cancelled search".to_string(),
            });
        }
        if let Some(max_time) = control.max_time
            && started.elapsed() >= max_time
        {
            break Some(Stop {
                status: SearchStatus::Partial,
                reason: "time bound reached".to_string(),
            });
        }
        if frontier.is_empty() {
            break None;
        }

        order_frontier(&mut frontier, control.order);
        let node = pop_frontier(&mut frontier, control.order);
        let Some(node) = node else {
            break None;
        };

        if branch_prunes(problem, &node.state, control.branch_and_bound, best_score) {
            metrics.pruned += 1;
            continue;
        }

        if let Some(output) = problem.finish(&node.state) {
            if let Err(stop) = enforce_result_capacity(&outputs, &control) {
                break Some(stop);
            }
            if let Err(stop) = charge(control.costs.emit, &control, &mut metrics) {
                break Some(stop);
            }
            metrics.emitted += 1;
            if let Some(score) = problem.output_score(&output) {
                best_score = Some(best_score.map_or(score, |best| best.min(score)));
            }
            outputs.push(output);
            metrics.result_count = outputs.len();
            if let Err(stop) = enforce_collection_limits(&frontier, &outputs, &control, &metrics) {
                break Some(stop);
            }
            continue;
        }

        let mut choices = Vec::new();
        if let Err(stop) = charge(control.costs.expand, &control, &mut metrics) {
            break Some(stop);
        }
        metrics.expanded += 1;
        problem.expand(&node.state, &mut choices);
        choices.sort();

        let mut children = Vec::new();
        let mut child_stop = None;
        for choice in choices {
            match problem.apply(&node.state, &choice) {
                SearchStep::Continue(state) => {
                    match propagate(problem, state, &control, &mut metrics) {
                        Ok(Some(state)) => {
                            if branch_prunes(problem, &state, control.branch_and_bound, best_score)
                            {
                                metrics.pruned += 1;
                                continue;
                            }
                            if let Err(stop) = charge(control.costs.score, &control, &mut metrics) {
                                child_stop = Some(stop);
                                break;
                            }
                            metrics.scored += 1;
                            children.push(Node {
                                priority: priority(
                                    problem,
                                    &state,
                                    node.sequence + 1,
                                    control.order,
                                ),
                                state,
                                sequence: next_sequence,
                            });
                            next_sequence += 1;
                        }
                        Ok(None) => metrics.pruned += 1,
                        Err(stop) => {
                            child_stop = Some(stop);
                            break;
                        }
                    }
                }
                SearchStep::Pruned { .. } | SearchStep::Infeasible { .. } => {
                    metrics.pruned += 1;
                }
            }
        }
        if let Some(stop) = child_stop {
            break Some(stop);
        }

        push_children(&mut frontier, children, control.order);
        if let SearchOrder::Beam { width } = control.order {
            order_frontier(&mut frontier, control.order);
            if frontier.len() > width {
                metrics.pruned += u64::try_from(frontier.len() - width).unwrap_or(u64::MAX);
                frontier.truncate(width);
            }
        }
        update_frontier(&mut metrics, frontier.len());
        if let Err(stop) = enforce_collection_limits(&frontier, &outputs, &control, &metrics) {
            break Some(stop);
        }
    };

    match stop {
        Some(stop) => finish(control, outputs, metrics, stop.status, Some(stop.reason)),
        None if outputs.is_empty() => {
            finish(control, outputs, metrics, SearchStatus::Infeasible, None)
        }
        None => finish(control, outputs, metrics, SearchStatus::Complete, None),
    }
}

fn propagate<P: SearchProblem>(
    problem: &P,
    state: P::State,
    control: &SearchControl,
    metrics: &mut Metrics,
) -> Result<Option<P::State>, Stop> {
    charge(control.costs.propagate, control, metrics)?;
    metrics.propagated += 1;
    match problem.propagate(state) {
        SearchStep::Continue(state) => Ok(Some(state)),
        SearchStep::Pruned { .. } | SearchStep::Infeasible { .. } => Ok(None),
    }
}

fn charge(cost: u64, control: &SearchControl, metrics: &mut Metrics) -> Result<(), Stop> {
    if let Some(limit) = control.max_work
        && metrics.work_used.saturating_add(cost) > limit
    {
        return Err(Stop {
            status: SearchStatus::Partial,
            reason: "work bound reached".to_string(),
        });
    }
    metrics.work_used = metrics.work_used.saturating_add(cost);
    Ok(())
}

fn enforce_result_capacity<Output>(
    outputs: &[Output],
    control: &SearchControl,
) -> Result<(), Stop> {
    if let Some(max_results) = control.max_results
        && outputs.len() >= max_results
    {
        return Err(Stop {
            status: SearchStatus::Partial,
            reason: "result bound reached".to_string(),
        });
    }
    Ok(())
}

fn enforce_collection_limits<State, Output>(
    frontier: &[Node<State>],
    outputs: &[Output],
    control: &SearchControl,
    metrics: &Metrics,
) -> Result<(), Stop> {
    enforce_result_capacity(outputs, control)?;
    if let Some(max_frontier) = control.max_frontier
        && frontier.len() > max_frontier
    {
        return Err(Stop {
            status: SearchStatus::Partial,
            reason: "frontier bound reached".to_string(),
        });
    }
    if let Some(max_memory_nodes) = control.max_memory_nodes
        && frontier.len().saturating_add(metrics.result_count) > max_memory_nodes
    {
        return Err(Stop {
            status: SearchStatus::Partial,
            reason: "memory node bound reached".to_string(),
        });
    }
    Ok(())
}

fn update_frontier(metrics: &mut Metrics, len: usize) {
    metrics.max_frontier = metrics.max_frontier.max(len);
}

fn priority<P: SearchProblem>(
    problem: &P,
    state: &P::State,
    depth: u64,
    order: SearchOrder,
) -> i64 {
    match order {
        SearchOrder::BestFirst => problem.score_state(state),
        SearchOrder::AStar | SearchOrder::Beam { .. } => problem
            .score_state(state)
            .saturating_add(problem.estimate_remaining(state)),
        SearchOrder::BreadthFirst => i64::try_from(depth).unwrap_or(i64::MAX),
        SearchOrder::DepthFirst => 0,
    }
}

fn branch_prunes<P: SearchProblem>(
    problem: &P,
    state: &P::State,
    enabled: bool,
    best_score: Option<i64>,
) -> bool {
    enabled
        && best_score
            .zip(problem.bound(state))
            .is_some_and(|(best, bound)| bound >= best)
}

fn pop_frontier<State>(frontier: &mut Vec<Node<State>>, order: SearchOrder) -> Option<Node<State>> {
    match order {
        SearchOrder::DepthFirst => frontier.pop(),
        SearchOrder::BreadthFirst
        | SearchOrder::BestFirst
        | SearchOrder::AStar
        | SearchOrder::Beam { .. } => {
            if frontier.is_empty() {
                None
            } else {
                Some(frontier.remove(0))
            }
        }
    }
}

fn push_children<State>(
    frontier: &mut Vec<Node<State>>,
    mut children: Vec<Node<State>>,
    order: SearchOrder,
) {
    if matches!(order, SearchOrder::DepthFirst) {
        children.reverse();
    }
    frontier.extend(children);
}

fn order_frontier<State>(frontier: &mut [Node<State>], order: SearchOrder) {
    match order {
        SearchOrder::DepthFirst | SearchOrder::BreadthFirst => {}
        SearchOrder::BestFirst | SearchOrder::AStar | SearchOrder::Beam { .. } => {
            frontier.sort_by(|left, right| compare_nodes(left, right));
        }
    }
}

fn compare_nodes<State>(left: &Node<State>, right: &Node<State>) -> Ordering {
    left.priority
        .cmp(&right.priority)
        .then_with(|| left.sequence.cmp(&right.sequence))
}

fn finish<Output: std::fmt::Debug>(
    control: SearchControl,
    outputs: Vec<Output>,
    mut metrics: Metrics,
    status: SearchStatus,
    reason: Option<String>,
) -> SearchRun<Output> {
    metrics.result_count = outputs.len();
    let output_material = outputs
        .iter()
        .map(|output| format!("{output:?}"))
        .collect::<Vec<_>>();
    let receipt = SearchReceipt::finalize(status, reason, metrics, &control, &output_material);
    SearchRun { outputs, receipt }
}
