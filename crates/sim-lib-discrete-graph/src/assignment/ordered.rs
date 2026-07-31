use super::{
    Assignment, AssignmentCertificate, AssignmentCost, AssignmentOperation, AssignmentPolicy,
    CostMatrix, add, certificate_error, less,
};
use crate::{AlgorithmReceipt, GraphError, control::WorkMeter};

#[derive(Clone)]
struct Candidate<C> {
    cost: C,
    action: Action,
}

#[derive(Copy, Clone)]
enum Action {
    Match(usize),
    Delete,
    Insert,
}

pub(super) fn solve<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    meter: &mut WorkMeter<'_>,
) -> Result<Assignment<C>, GraphError> {
    let suffix_costs = build_table(costs, policy, Some(meter))?;
    let operations = reconstruct(costs, policy, &suffix_costs)?;
    Ok(Assignment {
        operations,
        total_cost: suffix_costs[0][0].clone(),
        certificate: AssignmentCertificate::OrderPreserving { suffix_costs },
        receipt: empty_receipt(),
    })
}

fn build_table<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    mut meter: Option<&mut WorkMeter<'_>>,
) -> Result<Vec<Vec<C>>, GraphError> {
    let rows = costs.rows();
    let columns = costs.columns();
    let mut table = vec![vec![C::zero(); columns + 1]; rows + 1];

    for source in (0..rows).rev() {
        charge_cell(&mut meter)?;
        table[source][columns] = add(
            &policy.deletion_costs[source],
            &table[source + 1][columns],
            "ordered assignment deletion suffix",
        )?;
    }
    for target in (0..columns).rev() {
        charge_cell(&mut meter)?;
        table[rows][target] = add(
            &policy.insertion_costs[target],
            &table[rows][target + 1],
            "ordered assignment insertion suffix",
        )?;
    }
    for source in (0..rows).rev() {
        for target in (0..columns).rev() {
            charge_cell(&mut meter)?;
            let candidates = candidates(source, target, costs, policy, &table, &mut meter)?;
            let mut best: Option<Candidate<C>> = None;
            for candidate in candidates {
                let is_better = match &best {
                    Some(current) => less(
                        &candidate.cost,
                        &current.cost,
                        "ordered assignment candidate",
                    )?,
                    None => true,
                };
                if is_better {
                    best = Some(candidate);
                }
            }
            table[source][target] = best
                .expect("non-terminal assignment state has candidates")
                .cost;
        }
    }
    Ok(table)
}

fn candidates<C: AssignmentCost>(
    source: usize,
    target: usize,
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    table: &[Vec<C>],
    meter: &mut Option<&mut WorkMeter<'_>>,
) -> Result<Vec<Candidate<C>>, GraphError> {
    let mut candidates = Vec::new();
    let max_span = if policy.doubling_cost(source).is_some() {
        costs.columns() - target
    } else {
        1
    };
    let mut pair_total = C::zero();
    for span in 1..=max_span {
        charge_edge(meter)?;
        if !costs.allowed(source, target + span - 1) {
            break;
        }
        pair_total = add(
            &pair_total,
            costs.value(source, target + span - 1),
            "ordered assignment pair span",
        )?;
        if span > 1 {
            pair_total = add(
                &pair_total,
                policy
                    .doubling_cost(source)
                    .expect("span exceeds one only when doubling is enabled"),
                "ordered assignment doubling span",
            )?;
        }
        candidates.push(Candidate {
            cost: add(
                &pair_total,
                &table[source + 1][target + span],
                "ordered assignment match suffix",
            )?,
            action: Action::Match(span),
        });
    }
    candidates.push(Candidate {
        cost: {
            charge_edge(meter)?;
            add(
                &policy.deletion_costs[source],
                &table[source + 1][target],
                "ordered assignment deletion",
            )?
        },
        action: Action::Delete,
    });
    candidates.push(Candidate {
        cost: {
            charge_edge(meter)?;
            add(
                &policy.insertion_costs[target],
                &table[source][target + 1],
                "ordered assignment insertion",
            )?
        },
        action: Action::Insert,
    });
    Ok(candidates)
}

fn reconstruct<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    table: &[Vec<C>],
) -> Result<Vec<AssignmentOperation<C>>, GraphError> {
    let mut source = 0;
    let mut target = 0;
    let mut operations = Vec::new();
    while source < costs.rows() || target < costs.columns() {
        if source == costs.rows() {
            operations.push(AssignmentOperation::Insert {
                target,
                cost: policy.insertion_costs[target].clone(),
            });
            target += 1;
            continue;
        }
        if target == costs.columns() {
            operations.push(AssignmentOperation::Delete {
                source,
                cost: policy.deletion_costs[source].clone(),
            });
            source += 1;
            continue;
        }
        let choice = candidates(source, target, costs, policy, table, &mut None)?
            .into_iter()
            .find(|candidate| candidate.cost == table[source][target])
            .expect("table value comes from one candidate");
        match choice.action {
            Action::Match(span) => {
                operations.push(AssignmentOperation::Match {
                    source,
                    target,
                    cost: costs.value(source, target).clone(),
                });
                for offset in 1..span {
                    let doubled_target = target + offset;
                    operations.push(AssignmentOperation::Double {
                        source,
                        target: doubled_target,
                        cost: add(
                            costs.value(source, doubled_target),
                            policy
                                .doubling_cost(source)
                                .expect("doubling span has a configured cost"),
                            "ordered doubling operation",
                        )?,
                    });
                }
                source += 1;
                target += span;
            }
            Action::Delete => {
                operations.push(AssignmentOperation::Delete {
                    source,
                    cost: policy.deletion_costs[source].clone(),
                });
                source += 1;
            }
            Action::Insert => {
                operations.push(AssignmentOperation::Insert {
                    target,
                    cost: policy.insertion_costs[target].clone(),
                });
                target += 1;
            }
        }
    }
    Ok(operations)
}

pub(super) fn verify<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    assignment: &Assignment<C>,
    suffix_costs: &[Vec<C>],
) -> Result<(), GraphError> {
    if suffix_costs.len() != costs.rows() + 1
        || suffix_costs
            .iter()
            .any(|row| row.len() != costs.columns() + 1)
    {
        return certificate_error("ordered certificate table has the wrong dimensions");
    }
    let expected = build_table(costs, policy, None)?;
    if suffix_costs != expected {
        return certificate_error("ordered certificate violates the Bellman recurrence");
    }
    if assignment.total_cost != suffix_costs[0][0] {
        return certificate_error("ordered assignment does not equal the certified optimum");
    }
    let canonical = reconstruct(costs, policy, suffix_costs)?;
    if assignment.operations != canonical {
        return certificate_error("ordered assignment violates deterministic tie-breaking");
    }
    Ok(())
}

fn charge_cell(meter: &mut Option<&mut WorkMeter<'_>>) -> Result<(), GraphError> {
    if let Some(meter) = meter.as_deref_mut() {
        meter.cell()?;
    }
    Ok(())
}

fn charge_edge(meter: &mut Option<&mut WorkMeter<'_>>) -> Result<(), GraphError> {
    if let Some(meter) = meter.as_deref_mut() {
        meter.edge()?;
    }
    Ok(())
}

fn empty_receipt() -> AlgorithmReceipt {
    AlgorithmReceipt {
        work_used: 0,
        cells: 0,
        edges: 0,
        peak_memory_cells: 0,
        cell_work: 1,
        edge_work: 1,
    }
}
