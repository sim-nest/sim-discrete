use core::cmp::Ordering;

use super::{
    Assignment, AssignmentCertificate, AssignmentCost, AssignmentOperation, AssignmentPolicy,
    CostMatrix, DoublingPolicy, VoiceCrossingPolicy, add, certificate_error, flow, ordered,
};
use crate::{
    GraphError,
    cost::{compare, validate},
};

/// Verifies assignment feasibility, exact cost, and the supplied optimality
/// certificate.
pub fn verify_assignment<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    assignment: &Assignment<C>,
) -> Result<(), GraphError> {
    validate_inputs(costs, policy)?;
    assignment.receipt.validate()?;
    validate_operations(costs, policy, assignment)?;
    match (&policy.voice_crossing, &assignment.certificate) {
        (VoiceCrossingPolicy::Allow, AssignmentCertificate::MinCostFlow { potentials }) => {
            flow::verify(costs, policy, assignment, potentials)
        }
        (VoiceCrossingPolicy::Forbid, AssignmentCertificate::OrderPreserving { suffix_costs }) => {
            ordered::verify(costs, policy, assignment, suffix_costs)
        }
        _ => Err(GraphError::CertificateInvalid(
            "certificate kind does not match voice-crossing policy".to_owned(),
        )),
    }
}

pub(super) fn validate_inputs<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
) -> Result<(), GraphError> {
    if policy.insertion_costs.len() != costs.columns() {
        return Err(GraphError::InvalidAssignment(format!(
            "received {} insertion costs for {} targets",
            policy.insertion_costs.len(),
            costs.columns()
        )));
    }
    if policy.deletion_costs.len() != costs.rows() {
        return Err(GraphError::InvalidAssignment(format!(
            "received {} deletion costs for {} sources",
            policy.deletion_costs.len(),
            costs.rows()
        )));
    }
    if let DoublingPolicy::Allow { per_source } = &policy.doubling
        && per_source.len() != costs.rows()
    {
        return Err(GraphError::InvalidAssignment(format!(
            "received {} doubling costs for {} sources",
            per_source.len(),
            costs.rows()
        )));
    }
    let zero = C::zero();
    for cost in costs.values().iter().flatten() {
        validate_non_negative(cost, &zero, "assignment matrix")?;
    }
    for cost in &policy.insertion_costs {
        validate_non_negative(cost, &zero, "assignment insertion")?;
    }
    for cost in &policy.deletion_costs {
        validate_non_negative(cost, &zero, "assignment deletion")?;
    }
    if let DoublingPolicy::Allow { per_source } = &policy.doubling {
        for cost in per_source {
            validate_non_negative(cost, &zero, "assignment doubling")?;
        }
    }
    Ok(())
}

fn validate_operations<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    assignment: &Assignment<C>,
) -> Result<(), GraphError> {
    let mut source_targets = vec![Vec::new(); costs.rows()];
    let mut target_owner = vec![None; costs.columns()];
    let mut deleted = vec![false; costs.rows()];
    let mut total = C::zero();

    for operation in &assignment.operations {
        let operation_cost = match operation {
            AssignmentOperation::Match {
                source,
                target,
                cost,
            } => {
                register_pair(
                    *source,
                    *target,
                    false,
                    cost,
                    costs,
                    policy,
                    &mut source_targets,
                    &mut target_owner,
                )?;
                cost
            }
            AssignmentOperation::Double {
                source,
                target,
                cost,
            } => {
                register_pair(
                    *source,
                    *target,
                    true,
                    cost,
                    costs,
                    policy,
                    &mut source_targets,
                    &mut target_owner,
                )?;
                cost
            }
            AssignmentOperation::Insert { target, cost } => {
                if *target >= costs.columns() || target_owner[*target].replace(None).is_some() {
                    return certificate_error("target is assigned more than once or out of range");
                }
                if cost != &policy.insertion_costs[*target] {
                    return certificate_error("insertion operation has the wrong cost");
                }
                cost
            }
            AssignmentOperation::Delete { source, cost } => {
                if *source >= costs.rows() || core::mem::replace(&mut deleted[*source], true) {
                    return certificate_error("source is deleted more than once or out of range");
                }
                if cost != &policy.deletion_costs[*source] {
                    return certificate_error("deletion operation has the wrong cost");
                }
                cost
            }
        };
        total = add(&total, operation_cost, "assignment operation total")?;
    }

    if target_owner.iter().any(Option::is_none) {
        return certificate_error("not every target is assigned");
    }
    for source in 0..costs.rows() {
        let count = source_targets[source].len();
        if (count == 0) != deleted[source] {
            return certificate_error("source must be either used or deleted exactly once");
        }
        if count > 1 && matches!(policy.doubling, DoublingPolicy::Forbid) {
            return certificate_error("assignment doubles a source under a forbid policy");
        }
        if count > 0 {
            let matches = assignment
                .operations
                .iter()
                .filter(|operation| {
                    matches!(operation, AssignmentOperation::Match { source: found, .. } if *found == source)
                })
                .count();
            if matches != 1 {
                return certificate_error("each used source must have exactly one base match");
            }
        }
    }
    if policy.voice_crossing == VoiceCrossingPolicy::Forbid {
        let mut pairs = source_targets
            .iter()
            .enumerate()
            .flat_map(|(source, targets)| targets.iter().map(move |target| (source, *target)))
            .collect::<Vec<_>>();
        pairs.sort_unstable();
        if pairs.windows(2).any(|pair| pair[0].1 >= pair[1].1) {
            return certificate_error("assignment violates source/target order");
        }
    }
    if total != assignment.total_cost {
        return certificate_error("assignment total does not equal its operation costs");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn register_pair<C: AssignmentCost>(
    source: usize,
    target: usize,
    doubled: bool,
    operation_cost: &C,
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    source_targets: &mut [Vec<usize>],
    target_owner: &mut [Option<Option<usize>>],
) -> Result<(), GraphError> {
    if source >= costs.rows() || target >= costs.columns() {
        return certificate_error("match endpoint is out of range");
    }
    if !costs.allowed(source, target) {
        return certificate_error("match uses a forbidden assignment edge");
    }
    if target_owner[target].replace(Some(source)).is_some() {
        return certificate_error("target is assigned more than once");
    }
    let expected = if doubled {
        let doubling = policy
            .doubling_cost(source)
            .ok_or_else(|| GraphError::CertificateInvalid("doubling is forbidden".to_owned()))?;
        add(costs.value(source, target), doubling, "doubling operation")?
    } else {
        costs.value(source, target).clone()
    };
    if operation_cost != &expected {
        return certificate_error("match operation has the wrong cost");
    }
    source_targets[source].push(target);
    Ok(())
}

fn validate_non_negative<C: AssignmentCost>(
    cost: &C,
    zero: &C,
    context: &str,
) -> Result<(), GraphError> {
    validate(cost, context)?;
    if compare(cost, zero, context)? == Ordering::Less {
        return Err(GraphError::InvalidAssignment(
            "assignment costs must be non-negative".to_owned(),
        ));
    }
    Ok(())
}
