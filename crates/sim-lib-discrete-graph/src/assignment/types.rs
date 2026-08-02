use core::fmt::Debug;

use crate::{FiniteCost, GraphError};

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

    pub(super) fn values(&self) -> &[Option<C>] {
        &self.values
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
    pub receipt: crate::AlgorithmReceipt,
}
