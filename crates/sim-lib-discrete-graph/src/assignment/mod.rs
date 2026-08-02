//! Certified minimum-cost bipartite assignment.
//!
//! The unrestricted solver is a polynomial min-cost-flow reduction. The
//! no-crossing policy uses a polynomial sequence-alignment dynamic program,
//! because pairwise voice-order constraints are not ordinary edge costs.
//! Both paths return independently checkable optimality certificates.

// conformance: minimum-cost assignment verifies edits, deterministic ties, and optimality certificates.

mod flow;
mod ordered;
mod types;
mod verify;

use core::cmp::Ordering;

use crate::{
    AlgorithmControl, AlgorithmInterrupt, GraphError, NeverInterrupt,
    control::WorkMeter,
    cost::{add as add_cost, compare},
};

pub use types::{
    Assignment, AssignmentCertificate, AssignmentCost, AssignmentOperation, AssignmentPolicy,
    CostMatrix, DoublingPolicy, VoiceCrossingPolicy,
};
pub use verify::verify_assignment;

/// Finds a minimum-cost assignment under insertion, deletion, doubling, and
/// voice-crossing rules.
///
/// Ties are stable: the unrestricted solver uses canonical source/target edge
/// order, while the no-crossing solver prefers a shorter match span, then
/// deletion, then insertion at each equal-cost suffix.
pub fn min_cost_assignment<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: AssignmentPolicy<C>,
) -> Result<Assignment<C>, GraphError> {
    min_cost_assignment_with_control(costs, policy, &AlgorithmControl::default(), &NeverInterrupt)
}

/// Finds a minimum-cost assignment under explicit work and cancellation
/// control.
pub fn min_cost_assignment_with_control<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: AssignmentPolicy<C>,
    control: &AlgorithmControl,
    interrupt: &dyn AlgorithmInterrupt,
) -> Result<Assignment<C>, GraphError> {
    verify::validate_inputs(costs, &policy)?;
    let memory = assignment_memory_cells(costs)?;
    let mut meter = WorkMeter::new(control, interrupt, memory)?;
    let assignment = match policy.voice_crossing {
        VoiceCrossingPolicy::Allow => flow::solve(costs, &policy, &mut meter)?,
        VoiceCrossingPolicy::Forbid => ordered::solve(costs, &policy, &mut meter)?,
    };
    let assignment = Assignment {
        receipt: meter.finish(),
        ..assignment
    };
    verify_assignment(costs, &policy, &assignment)?;
    Ok(assignment)
}

pub(super) fn add<C: AssignmentCost>(left: &C, right: &C, context: &str) -> Result<C, GraphError> {
    add_cost(left, right, context)
}

pub(super) fn sub<C: AssignmentCost>(left: &C, right: &C, context: &str) -> Result<C, GraphError> {
    left.checked_sub(right)
        .ok_or_else(|| GraphError::WeightOverflow(context.to_owned()))
}

pub(super) fn certificate_error<T>(message: &str) -> Result<T, GraphError> {
    Err(GraphError::CertificateInvalid(message.to_owned()))
}

pub(super) fn less<C: AssignmentCost>(
    left: &C,
    right: &C,
    context: &str,
) -> Result<bool, GraphError> {
    Ok(compare(left, right, context)? == Ordering::Less)
}

fn assignment_memory_cells<C>(costs: &CostMatrix<C>) -> Result<usize, GraphError> {
    costs
        .rows()
        .checked_add(1)
        .and_then(|rows| {
            costs
                .columns()
                .checked_add(1)
                .and_then(|columns| rows.checked_mul(columns))
        })
        .ok_or_else(|| GraphError::InvalidAssignment("assignment dimensions overflow".to_owned()))
}

#[cfg(test)]
mod tests;
