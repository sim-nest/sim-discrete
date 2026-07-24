//! Search controls and work charging policy.

use std::time::Duration;

/// Deterministic frontier policy used by [`crate::solve`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchOrder {
    /// Depth-first search with choices visited in sorted order.
    DepthFirst,
    /// Breadth-first search with choices visited in sorted order.
    BreadthFirst,
    /// Best-first search ordered by `SearchProblem::score_state`.
    BestFirst,
    /// A-star search ordered by `score_state + estimate_remaining`.
    AStar,
    /// Beam search ordered like A-star while retaining at most `width` frontier
    /// nodes after each expansion.
    Beam {
        /// Maximum number of frontier nodes retained by the beam.
        width: usize,
    },
}

impl SearchOrder {
    /// Stable label used in receipts and policy digests.
    pub fn label(self) -> &'static str {
        match self {
            Self::DepthFirst => "depth-first",
            Self::BreadthFirst => "breadth-first",
            Self::BestFirst => "best-first",
            Self::AStar => "a-star",
            Self::Beam { .. } => "beam",
        }
    }

    pub(crate) fn policy_material(self) -> String {
        match self {
            Self::Beam { width } => format!("order=beam,width={width}"),
            other => format!("order={}", other.label()),
        }
    }
}

/// Work charges applied by the engine for each observable operation class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkCosts {
    /// Cost charged before expanding a state's choices.
    pub expand: u64,
    /// Cost charged before scoring or prioritizing a child state.
    pub score: u64,
    /// Cost charged before running propagation on a child state.
    pub propagate: u64,
    /// Cost charged before emitting a finished output.
    pub emit: u64,
}

impl Default for WorkCosts {
    fn default() -> Self {
        Self {
            expand: 1,
            score: 1,
            propagate: 1,
            emit: 1,
        }
    }
}

/// Bounds, ordering, seed, and accounting policy for one search run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchControl {
    /// Deterministic frontier ordering policy.
    pub order: SearchOrder,
    /// Caller-supplied seed recorded in the receipt digest.
    pub seed: u64,
    /// Optional maximum charged work.
    pub max_work: Option<u64>,
    /// Optional maximum number of emitted outputs.
    pub max_results: Option<usize>,
    /// Optional maximum frontier length.
    pub max_frontier: Option<usize>,
    /// Optional maximum `frontier + results` node count.
    pub max_memory_nodes: Option<usize>,
    /// Optional wall-clock deadline for the run.
    pub max_time: Option<Duration>,
    /// Whether lower-bound pruning uses the best emitted output score.
    pub branch_and_bound: bool,
    /// Per-operation work charges.
    pub costs: WorkCosts,
}

impl Default for SearchControl {
    fn default() -> Self {
        Self {
            order: SearchOrder::DepthFirst,
            seed: 0,
            max_work: None,
            max_results: None,
            max_frontier: None,
            max_memory_nodes: None,
            max_time: None,
            branch_and_bound: false,
            costs: WorkCosts::default(),
        }
    }
}

impl SearchControl {
    /// Return a copy with a different deterministic frontier order.
    pub fn with_order(mut self, order: SearchOrder) -> Self {
        self.order = order;
        self
    }

    /// Return a copy with a different receipt seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Return a copy with a maximum charged-work bound.
    pub fn with_max_work(mut self, max_work: u64) -> Self {
        self.max_work = Some(max_work);
        self
    }

    /// Return a copy with a maximum emitted-result bound.
    pub fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = Some(max_results);
        self
    }

    /// Return a copy with a maximum frontier bound.
    pub fn with_max_frontier(mut self, max_frontier: usize) -> Self {
        self.max_frontier = Some(max_frontier);
        self
    }

    /// Return a copy with a maximum `frontier + results` bound.
    pub fn with_max_memory_nodes(mut self, max_memory_nodes: usize) -> Self {
        self.max_memory_nodes = Some(max_memory_nodes);
        self
    }

    /// Return a copy with a wall-clock deadline.
    pub fn with_max_time(mut self, max_time: Duration) -> Self {
        self.max_time = Some(max_time);
        self
    }

    /// Return a copy with branch-and-bound pruning enabled or disabled.
    pub fn with_branch_and_bound(mut self, enabled: bool) -> Self {
        self.branch_and_bound = enabled;
        self
    }

    /// Return a copy with explicit per-operation work costs.
    pub fn with_costs(mut self, costs: WorkCosts) -> Self {
        self.costs = costs;
        self
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if matches!(self.order, SearchOrder::Beam { width: 0 }) {
            return Err("beam width must be greater than zero".to_string());
        }
        if self.costs.expand == 0
            || self.costs.score == 0
            || self.costs.propagate == 0
            || self.costs.emit == 0
        {
            return Err("work costs must be positive".to_string());
        }
        Ok(())
    }

    pub(crate) fn policy_material(&self) -> String {
        format!(
            "{};seed={};max_work={};max_results={};max_frontier={};max_memory_nodes={};max_time_ns={};branch_and_bound={};costs={},{},{},{}",
            self.order.policy_material(),
            self.seed,
            option_u64(self.max_work),
            option_usize(self.max_results),
            option_usize(self.max_frontier),
            option_usize(self.max_memory_nodes),
            self.max_time
                .map(|duration| duration.as_nanos().to_string())
                .unwrap_or_else(|| "none".to_string()),
            self.branch_and_bound,
            self.costs.expand,
            self.costs.score,
            self.costs.propagate,
            self.costs.emit,
        )
    }
}

fn option_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn option_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}
