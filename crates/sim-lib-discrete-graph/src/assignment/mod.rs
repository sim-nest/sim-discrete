//! Certified minimum-cost bipartite assignment.
//!
//! The unrestricted solver is a polynomial min-cost-flow reduction. The
//! no-crossing policy uses a polynomial sequence-alignment dynamic program,
//! because pairwise voice-order constraints are not ordinary edge costs.
//! Both paths return independently checkable optimality certificates.

// conformance: minimum-cost assignment verifies edits, deterministic ties, and optimality certificates.

mod flow;
mod ordered;

use core::{cmp::Ordering, fmt::Debug};

use crate::{
    AlgorithmControl, AlgorithmInterrupt, AlgorithmReceipt, FiniteCost, GraphError, NeverInterrupt,
    control::WorkMeter,
    cost::{add as add_cost, compare, validate},
};

/// Additive, ordered cost used by certified assignment.
///
/// Assignment needs checked addition for totals and checked subtraction for
/// min-cost-flow residual edges and dual certificates. Implementations must
/// provide exact arithmetic: saturating or wrapping implementations violate the
/// certificate contract.
pub trait AssignmentCost: FiniteCost {
    /// Exact checked subtraction.
    fn checked_sub(&self, rhs: &Self) -> Option<Self>;
}

macro_rules! integer_assignment_cost {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl AssignmentCost for $ty {
                fn checked_sub(&self, rhs: &Self) -> Option<Self> {
                    (*self).checked_sub(*rhs)
                }
            }
        )+
    };
}

integer_assignment_cost!(i32, i64, i128, isize);

macro_rules! float_assignment_cost {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl AssignmentCost for $ty {
                fn checked_sub(&self, rhs: &Self) -> Option<Self> {
                    let value = *self - *rhs;
                    value.is_finite().then_some(value)
                }
            }
        )+
    };
}

float_assignment_cost!(f32, f64);

/// Dense row-major costs between source and target items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CostMatrix<C> {
    rows: usize,
    columns: usize,
    values: Vec<Option<C>>,
}

impl<C> CostMatrix<C> {
    /// Builds a `rows` by `columns` matrix from row-major `values`.
    pub fn new(rows: usize, columns: usize, values: Vec<C>) -> Result<Self, GraphError> {
        let expected = rows.checked_mul(columns).ok_or_else(|| {
            GraphError::InvalidAssignment("cost matrix dimensions overflow".to_owned())
        })?;
        if values.len() != expected {
            return Err(GraphError::InvalidAssignment(format!(
                "cost matrix has {} values, expected {expected}",
                values.len()
            )));
        }
        Ok(Self {
            rows,
            columns,
            values: values.into_iter().map(Some).collect(),
        })
    }

    /// Builds a matrix whose `None` entries are forbidden assignment edges.
    pub fn from_optional(
        rows: usize,
        columns: usize,
        values: Vec<Option<C>>,
    ) -> Result<Self, GraphError> {
        let expected = rows.checked_mul(columns).ok_or_else(|| {
            GraphError::InvalidAssignment("cost matrix dimensions overflow".to_owned())
        })?;
        if values.len() != expected {
            return Err(GraphError::InvalidAssignment(format!(
                "cost matrix has {} values, expected {expected}",
                values.len()
            )));
        }
        Ok(Self {
            rows,
            columns,
            values,
        })
    }

    /// Number of source rows.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of target columns.
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Returns the allowed cost at `(source, target)`.
    ///
    /// Returns `None` for both forbidden and out-of-range edges. Use
    /// [`CostMatrix::is_forbidden`] when that distinction matters.
    pub fn get(&self, source: usize, target: usize) -> Option<&C> {
        if source >= self.rows || target >= self.columns {
            return None;
        }
        self.values
            .get(source * self.columns + target)
            .and_then(Option::as_ref)
    }

    /// Marks an in-range edge forbidden and returns its previous cost.
    pub fn forbid(&mut self, source: usize, target: usize) -> Result<Option<C>, GraphError> {
        let index = self.index(source, target)?;
        Ok(self.values[index].take())
    }

    /// Sets or restores an in-range edge cost.
    pub fn allow(&mut self, source: usize, target: usize, cost: C) -> Result<(), GraphError> {
        let index = self.index(source, target)?;
        self.values[index] = Some(cost);
        Ok(())
    }

    /// Whether an in-range edge is explicitly forbidden.
    pub fn is_forbidden(&self, source: usize, target: usize) -> Result<bool, GraphError> {
        let index = self.index(source, target)?;
        Ok(self.values[index].is_none())
    }

    pub(super) fn value(&self, source: usize, target: usize) -> &C {
        self.values[source * self.columns + target]
            .as_ref()
            .expect("algorithm requests only allowed assignment edges")
    }

    pub(super) fn allowed(&self, source: usize, target: usize) -> bool {
        self.values[source * self.columns + target].is_some()
    }

    fn index(&self, source: usize, target: usize) -> Result<usize, GraphError> {
        if source >= self.rows || target >= self.columns {
            return Err(GraphError::InvalidAssignment(format!(
                "assignment edge ({source}, {target}) is outside {} by {} matrix",
                self.rows, self.columns
            )));
        }
        Ok(source * self.columns + target)
    }
}

impl<C> TryFrom<Vec<Vec<C>>> for CostMatrix<C> {
    type Error = GraphError;

    fn try_from(rows: Vec<Vec<C>>) -> Result<Self, Self::Error> {
        let columns = rows.first().map_or(0, Vec::len);
        if rows.iter().any(|row| row.len() != columns) {
            return Err(GraphError::InvalidAssignment(
                "cost matrix rows have different lengths".to_owned(),
            ));
        }
        let row_count = rows.len();
        Self::new(row_count, columns, rows.into_iter().flatten().collect())
    }
}

/// Whether one source may supply multiple targets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DoublingPolicy<C> {
    /// Every source may match at most one target.
    Forbid,
    /// A source may supply further targets at the corresponding per-source
    /// incremental cost.
    Allow {
        /// Cost charged for every target after the first supplied by a source.
        per_source: Vec<C>,
    },
}

/// Policy for assignments that reverse the order of two source voices.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VoiceCrossingPolicy {
    /// Source-to-target edges may cross.
    Allow,
    /// Matches must preserve source and target order.
    Forbid,
}

/// Costs and structural rules for one assignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssignmentPolicy<C> {
    /// Cost for each target that enters without a source.
    pub insertion_costs: Vec<C>,
    /// Cost for each source that leaves without a target.
    pub deletion_costs: Vec<C>,
    /// Whether and at what cost sources may be doubled.
    pub doubling: DoublingPolicy<C>,
    /// Whether source voices may cross.
    pub voice_crossing: VoiceCrossingPolicy,
}

impl<C> AssignmentPolicy<C> {
    /// Builds a policy with explicit insertion/deletion costs, no doubling, and
    /// crossings allowed.
    pub fn new(insertion_costs: Vec<C>, deletion_costs: Vec<C>) -> Self {
        Self {
            insertion_costs,
            deletion_costs,
            doubling: DoublingPolicy::Forbid,
            voice_crossing: VoiceCrossingPolicy::Allow,
        }
    }

    /// Enables source doubling at a per-source incremental cost.
    pub fn with_doubling(mut self, per_source: Vec<C>) -> Self {
        self.doubling = DoublingPolicy::Allow { per_source };
        self
    }

    /// Sets the voice-crossing policy.
    pub fn with_voice_crossing(mut self, policy: VoiceCrossingPolicy) -> Self {
        self.voice_crossing = policy;
        self
    }

    pub(super) fn doubling_cost(&self, source: usize) -> Option<&C> {
        match &self.doubling {
            DoublingPolicy::Forbid => None,
            DoublingPolicy::Allow { per_source } => per_source.get(source),
        }
    }
}

/// One edit or correspondence in a complete bipartite assignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssignmentOperation<C> {
    /// One source is paired with one target.
    Match {
        /// Source row.
        source: usize,
        /// Target column.
        target: usize,
        /// Pair cost from the matrix.
        cost: C,
    },
    /// A source supplies an additional target.
    Double {
        /// Reused source row.
        source: usize,
        /// Additional target column.
        target: usize,
        /// Pair cost plus the configured doubling cost.
        cost: C,
    },
    /// A target enters without a source.
    Insert {
        /// Target column.
        target: usize,
        /// Configured insertion cost.
        cost: C,
    },
    /// A source leaves without a target.
    Delete {
        /// Source row.
        source: usize,
        /// Configured deletion cost.
        cost: C,
    },
}

/// Optimality witness for one of the two assignment solvers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssignmentCertificate<C> {
    /// Feasible min-cost flow plus node potentials proving every residual edge
    /// has non-negative reduced cost.
    MinCostFlow {
        /// Dual potential for every node in the canonical assignment network.
        potentials: Vec<C>,
    },
    /// Bellman table proving every suffix has the stated minimum cost under the
    /// no-crossing recurrence.
    OrderPreserving {
        /// `(source, target)` suffix costs, including the terminal row/column.
        suffix_costs: Vec<Vec<C>>,
    },
}

/// A complete minimum-cost assignment and its optimality certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment<C> {
    /// Operations in stable source/target order.
    pub operations: Vec<AssignmentOperation<C>>,
    /// Exact sum of operation costs.
    pub total_cost: C,
    /// Independently checkable optimality witness.
    pub certificate: AssignmentCertificate<C>,
    /// Deterministic cell/edge work accounting for the solve.
    pub receipt: AlgorithmReceipt,
}

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
    validate_inputs(costs, &policy)?;
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

fn validate_inputs<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
) -> Result<(), GraphError> {
    if policy.insertion_costs.len() != costs.columns {
        return Err(GraphError::InvalidAssignment(format!(
            "received {} insertion costs for {} targets",
            policy.insertion_costs.len(),
            costs.columns
        )));
    }
    if policy.deletion_costs.len() != costs.rows {
        return Err(GraphError::InvalidAssignment(format!(
            "received {} deletion costs for {} sources",
            policy.deletion_costs.len(),
            costs.rows
        )));
    }
    if let DoublingPolicy::Allow { per_source } = &policy.doubling
        && per_source.len() != costs.rows
    {
        return Err(GraphError::InvalidAssignment(format!(
            "received {} doubling costs for {} sources",
            per_source.len(),
            costs.rows
        )));
    }
    let zero = C::zero();
    for cost in costs.values.iter().flatten() {
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
    let mut source_targets = vec![Vec::new(); costs.rows];
    let mut target_owner = vec![None; costs.columns];
    let mut deleted = vec![false; costs.rows];
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
                if *target >= costs.columns || target_owner[*target].replace(None).is_some() {
                    return certificate_error("target is assigned more than once or out of range");
                }
                if cost != &policy.insertion_costs[*target] {
                    return certificate_error("insertion operation has the wrong cost");
                }
                cost
            }
            AssignmentOperation::Delete { source, cost } => {
                if *source >= costs.rows || core::mem::replace(&mut deleted[*source], true) {
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
    for source in 0..costs.rows {
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
    if source >= costs.rows || target >= costs.columns {
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

fn assignment_memory_cells<C>(costs: &CostMatrix<C>) -> Result<usize, GraphError> {
    costs
        .rows
        .checked_add(1)
        .and_then(|rows| {
            costs
                .columns
                .checked_add(1)
                .and_then(|columns| rows.checked_mul(columns))
        })
        .ok_or_else(|| GraphError::InvalidAssignment("assignment dimensions overflow".to_owned()))
}

#[cfg(test)]
mod tests;
