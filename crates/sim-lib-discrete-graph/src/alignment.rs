//! Certified dynamic-time-warp and edit alignment.

use core::cmp::Ordering;

use crate::{
    AlgorithmControl, AlgorithmInterrupt, AlgorithmReceipt, FiniteCost, GraphError, NeverInterrupt,
    control::WorkMeter,
    cost::{add, compare, validate},
};

mod verify;

pub use verify::verify_alignment;

/// Window limiting which prefix-pair cells an alignment may visit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlignmentWindow {
    /// Evaluate the full prefix grid.
    #[default]
    Unbounded,
    /// Permit cells whose prefix indices differ by at most `radius`.
    Radius(usize),
}

impl AlignmentWindow {
    fn contains(self, left: usize, right: usize) -> bool {
        match self {
            Self::Unbounded => true,
            Self::Radius(radius) => left.abs_diff(right) <= radius,
        }
    }
}

/// Endpoint semantics for sequence alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlignmentBoundary {
    /// Align both complete sequences.
    #[default]
    Global,
    /// Align the complete left query to a contiguous region of the right
    /// sequence. Right-side prefix and suffix costs are free.
    Subsequence,
}

/// Whether to retain the full proof table or only the final rolling row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlignmentMemory {
    /// Retain backpointers and return the full alignment path.
    #[default]
    Full,
    /// Retain two score rows while solving and return score-only evidence.
    RollingScoreOnly,
}

/// Costs for advancing only one side of an edit alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GapPolicy<C> {
    /// Cost of consuming a left item without a right item.
    pub delete: C,
    /// Cost of consuming a right item without a left item.
    pub insert: C,
}

impl<C> GapPolicy<C> {
    /// Builds a gap policy with explicit deletion and insertion costs.
    pub fn new(delete: C, insert: C) -> Self {
        Self { delete, insert }
    }
}

/// Window, gap, boundary, and memory policy for dynamic alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DtwPolicy<C> {
    /// Admissible prefix-grid cells.
    pub window: AlignmentWindow,
    /// One-sided step costs.
    pub gaps: GapPolicy<C>,
    /// Required start/end coverage.
    pub boundary: AlignmentBoundary,
    /// Retained proof material.
    pub memory: AlignmentMemory,
}

impl<C> DtwPolicy<C> {
    /// Builds a global, unbounded, full-memory alignment policy.
    pub fn new(gaps: GapPolicy<C>) -> Self {
        Self {
            window: AlignmentWindow::Unbounded,
            gaps,
            boundary: AlignmentBoundary::Global,
            memory: AlignmentMemory::Full,
        }
    }

    /// Returns a copy with a different alignment window.
    pub fn with_window(mut self, window: AlignmentWindow) -> Self {
        self.window = window;
        self
    }

    /// Returns a copy with different endpoint semantics.
    pub fn with_boundary(mut self, boundary: AlignmentBoundary) -> Self {
        self.boundary = boundary;
        self
    }

    /// Returns a copy with a different memory policy.
    pub fn with_memory(mut self, memory: AlignmentMemory) -> Self {
        self.memory = memory;
        self
    }
}

/// Stable predecessor move in a full alignment certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentMove {
    /// Consume one item from each sequence.
    Match,
    /// Consume one left item.
    Delete,
    /// Consume one right item.
    Insert,
}

/// One reachable prefix-grid cell in a full alignment certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlignmentCell<C> {
    /// Minimum cost of reaching this cell.
    pub total_cost: C,
    /// Stable predecessor move, or `None` at a permitted free start.
    pub predecessor: Option<AlignmentMove>,
    /// Cost charged by the predecessor move.
    pub step_cost: Option<C>,
}

/// Optimality evidence retained under the selected memory policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlignmentCertificate<C> {
    /// Full prefix-grid Bellman table with backpointers.
    Full {
        /// Row-major `(left prefix, right prefix)` cells.
        cells: Vec<Vec<Option<AlignmentCell<C>>>>,
    },
    /// Final score row reproduced by rolling-memory evaluation.
    Rolling {
        /// Cost at every right prefix after consuming the complete left input.
        final_row: Vec<Option<C>>,
        /// Stable selected right endpoint.
        endpoint: usize,
    },
}

/// One operation in a full alignment path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlignmentStep<C> {
    /// Pair one left and one right item.
    Match {
        /// Left input index.
        left: usize,
        /// Right input index.
        right: usize,
        /// Local pair cost.
        cost: C,
    },
    /// Consume an unmatched left item.
    Delete {
        /// Left input index.
        left: usize,
        /// Gap cost.
        cost: C,
    },
    /// Consume an unmatched right item.
    Insert {
        /// Right input index.
        right: usize,
        /// Gap cost.
        cost: C,
    },
}

/// Minimum-cost alignment plus proof and deterministic accounting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alignment<C> {
    /// Exact minimum score.
    pub score: C,
    /// Stable full path, absent in rolling score-only mode.
    pub steps: Option<Vec<AlignmentStep<C>>>,
    /// Proof material selected by the memory policy.
    pub certificate: AlignmentCertificate<C>,
    /// Cell/edge work accounting.
    pub receipt: AlgorithmReceipt,
}

/// Aligns two sequences under a window, gap, boundary, and memory policy.
pub fn dynamic_time_warp<T, C: FiniteCost>(
    left: &[T],
    right: &[T],
    local_cost: impl Fn(&T, &T) -> C,
    policy: DtwPolicy<C>,
) -> Result<Alignment<C>, GraphError> {
    dynamic_time_warp_with_control(
        left,
        right,
        local_cost,
        policy,
        &AlgorithmControl::default(),
        &NeverInterrupt,
    )
}

/// Aligns two sequences under explicit work and cancellation control.
pub fn dynamic_time_warp_with_control<T, C: FiniteCost>(
    left: &[T],
    right: &[T],
    local_cost: impl Fn(&T, &T) -> C,
    policy: DtwPolicy<C>,
    control: &AlgorithmControl,
    interrupt: &dyn AlgorithmInterrupt,
) -> Result<Alignment<C>, GraphError> {
    validate_policy(&policy)?;
    let memory = peak_memory(left.len(), right.len(), policy.memory)?;
    let mut meter = WorkMeter::new(control, interrupt, memory)?;
    let computed = match policy.memory {
        AlignmentMemory::Full => {
            let (cells, stats) = full_table(left, right, &local_cost, &policy, Some(&mut meter))?;
            let endpoint = select_endpoint(&cells, policy.boundary)?;
            let score = cells[left.len()][endpoint]
                .as_ref()
                .expect("selected endpoint is reachable")
                .total_cost
                .clone();
            let steps = reconstruct(&cells, left.len(), endpoint, policy.boundary)?;
            Computed {
                score,
                steps: Some(steps),
                certificate: AlignmentCertificate::Full { cells },
                stats,
            }
        }
        AlignmentMemory::RollingScoreOnly => {
            let (final_row, stats) =
                rolling_row(left, right, &local_cost, &policy, Some(&mut meter))?;
            let endpoint = select_rolling_endpoint(&final_row, policy.boundary, right.len())?;
            let score = final_row[endpoint]
                .as_ref()
                .expect("selected endpoint is reachable")
                .clone();
            Computed {
                score,
                steps: None,
                certificate: AlignmentCertificate::Rolling {
                    final_row,
                    endpoint,
                },
                stats,
            }
        }
    };
    let receipt = meter.finish();
    debug_assert_eq!(receipt.cells, computed.stats.cells);
    debug_assert_eq!(receipt.edges, computed.stats.edges);
    Ok(Alignment {
        score: computed.score,
        steps: computed.steps,
        certificate: computed.certificate,
        receipt,
    })
}

struct Computed<C> {
    score: C,
    steps: Option<Vec<AlignmentStep<C>>>,
    certificate: AlignmentCertificate<C>,
    stats: Stats,
}

#[derive(Clone, Copy, Debug, Default)]
struct Stats {
    cells: u64,
    edges: u64,
}

type Table<C> = Vec<Vec<Option<AlignmentCell<C>>>>;

fn full_table<T, C: FiniteCost>(
    left: &[T],
    right: &[T],
    local_cost: &impl Fn(&T, &T) -> C,
    policy: &DtwPolicy<C>,
    mut meter: Option<&mut WorkMeter<'_>>,
) -> Result<(Table<C>, Stats), GraphError> {
    let rows = left
        .len()
        .checked_add(1)
        .ok_or_else(|| GraphError::WeightOverflow("alignment rows".to_owned()))?;
    let columns = right
        .len()
        .checked_add(1)
        .ok_or_else(|| GraphError::WeightOverflow("alignment columns".to_owned()))?;
    let mut cells = vec![vec![None; columns]; rows];
    let mut stats = Stats::default();
    for i in 0..rows {
        for j in 0..columns {
            if !policy.window.contains(i, j) {
                continue;
            }
            charge_cell(&mut meter, &mut stats)?;
            cells[i][j] = compute_cell(
                i,
                j,
                left,
                right,
                local_cost,
                policy,
                |row, column| cells[row][column].clone(),
                &mut meter,
                &mut stats,
            )?;
        }
    }
    Ok((cells, stats))
}

#[allow(clippy::too_many_arguments)]
fn compute_cell<C: FiniteCost, T>(
    i: usize,
    j: usize,
    left: &[T],
    right: &[T],
    local_cost: &impl Fn(&T, &T) -> C,
    policy: &DtwPolicy<C>,
    lookup: impl Fn(usize, usize) -> Option<AlignmentCell<C>>,
    meter: &mut Option<&mut WorkMeter<'_>>,
    stats: &mut Stats,
) -> Result<Option<AlignmentCell<C>>, GraphError> {
    if i == 0 && (j == 0 || policy.boundary == AlignmentBoundary::Subsequence) {
        return Ok(Some(AlignmentCell {
            total_cost: C::zero(),
            predecessor: None,
            step_cost: None,
        }));
    }
    let mut best: Option<(C, AlignmentMove, C)> = None;
    if i > 0 && j > 0 {
        charge_edge(meter, stats)?;
        if let Some(previous) = lookup(i - 1, j - 1) {
            let cost = local_cost(&left[i - 1], &right[j - 1]);
            validate_non_negative(&cost, "alignment local cost")?;
            let total = add(&previous.total_cost, &cost, "alignment match")?;
            choose(&mut best, total, AlignmentMove::Match, cost)?;
        }
    }
    if i > 0 {
        charge_edge(meter, stats)?;
        if let Some(previous) = lookup(i - 1, j) {
            let total = add(
                &previous.total_cost,
                &policy.gaps.delete,
                "alignment deletion",
            )?;
            choose(
                &mut best,
                total,
                AlignmentMove::Delete,
                policy.gaps.delete.clone(),
            )?;
        }
    }
    if j > 0 {
        charge_edge(meter, stats)?;
        if let Some(previous) = lookup(i, j - 1) {
            let total = add(
                &previous.total_cost,
                &policy.gaps.insert,
                "alignment insertion",
            )?;
            choose(
                &mut best,
                total,
                AlignmentMove::Insert,
                policy.gaps.insert.clone(),
            )?;
        }
    }
    Ok(
        best.map(|(total_cost, predecessor, step_cost)| AlignmentCell {
            total_cost,
            predecessor: Some(predecessor),
            step_cost: Some(step_cost),
        }),
    )
}

fn rolling_row<T, C: FiniteCost>(
    left: &[T],
    right: &[T],
    local_cost: &impl Fn(&T, &T) -> C,
    policy: &DtwPolicy<C>,
    mut meter: Option<&mut WorkMeter<'_>>,
) -> Result<(Vec<Option<C>>, Stats), GraphError> {
    let columns = right
        .len()
        .checked_add(1)
        .ok_or_else(|| GraphError::WeightOverflow("alignment columns".to_owned()))?;
    let mut previous = vec![None; columns];
    let mut stats = Stats::default();
    for i in 0..=left.len() {
        let mut current = vec![None; columns];
        for j in 0..columns {
            if !policy.window.contains(i, j) {
                continue;
            }
            charge_cell(&mut meter, &mut stats)?;
            if i == 0 && (j == 0 || policy.boundary == AlignmentBoundary::Subsequence) {
                current[j] = Some(C::zero());
                continue;
            }
            let mut best: Option<C> = None;
            if i > 0 && j > 0 {
                charge_edge(&mut meter, &mut stats)?;
                if let Some(prior) = &previous[j - 1] {
                    let cost = local_cost(&left[i - 1], &right[j - 1]);
                    validate_non_negative(&cost, "alignment local cost")?;
                    choose_score(&mut best, add(prior, &cost, "alignment match")?)?;
                }
            }
            if i > 0 {
                charge_edge(&mut meter, &mut stats)?;
                if let Some(prior) = &previous[j] {
                    choose_score(
                        &mut best,
                        add(prior, &policy.gaps.delete, "alignment deletion")?,
                    )?;
                }
            }
            if j > 0 {
                charge_edge(&mut meter, &mut stats)?;
                if let Some(prior) = &current[j - 1] {
                    choose_score(
                        &mut best,
                        add(prior, &policy.gaps.insert, "alignment insertion")?,
                    )?;
                }
            }
            current[j] = best;
        }
        previous = current;
    }
    Ok((previous, stats))
}

fn choose<C: FiniteCost>(
    best: &mut Option<(C, AlignmentMove, C)>,
    total: C,
    movement: AlignmentMove,
    step_cost: C,
) -> Result<(), GraphError> {
    let replace = match best {
        Some((current, _, _)) => {
            compare(&total, current, "alignment candidate ordering")? == Ordering::Less
        }
        None => true,
    };
    if replace {
        *best = Some((total, movement, step_cost));
    }
    Ok(())
}

fn choose_score<C: FiniteCost>(best: &mut Option<C>, total: C) -> Result<(), GraphError> {
    let replace = match best {
        Some(current) => {
            compare(&total, current, "alignment candidate ordering")? == Ordering::Less
        }
        None => true,
    };
    if replace {
        *best = Some(total);
    }
    Ok(())
}

fn select_endpoint<C: FiniteCost>(
    cells: &Table<C>,
    boundary: AlignmentBoundary,
) -> Result<usize, GraphError> {
    let final_row = cells.last().expect("alignment table has a prefix row");
    select_cell_endpoint(final_row, boundary)
}

fn select_cell_endpoint<C: FiniteCost>(
    row: &[Option<AlignmentCell<C>>],
    boundary: AlignmentBoundary,
) -> Result<usize, GraphError> {
    match boundary {
        AlignmentBoundary::Global => row
            .len()
            .checked_sub(1)
            .filter(|endpoint| row[*endpoint].is_some())
            .ok_or(GraphError::Disconnected),
        AlignmentBoundary::Subsequence => {
            let mut best: Option<(usize, &C)> = None;
            for (index, cell) in row.iter().enumerate() {
                let Some(cell) = cell else {
                    continue;
                };
                let replace = match best {
                    Some((_, cost)) => {
                        compare(&cell.total_cost, cost, "alignment endpoint ordering")?
                            == Ordering::Less
                    }
                    None => true,
                };
                if replace {
                    best = Some((index, &cell.total_cost));
                }
            }
            best.map(|(index, _)| index).ok_or(GraphError::Disconnected)
        }
    }
}

fn select_rolling_endpoint<C: FiniteCost>(
    row: &[Option<C>],
    boundary: AlignmentBoundary,
    right_len: usize,
) -> Result<usize, GraphError> {
    match boundary {
        AlignmentBoundary::Global => row
            .get(right_len)
            .and_then(Option::as_ref)
            .map(|_| right_len)
            .ok_or(GraphError::Disconnected),
        AlignmentBoundary::Subsequence => {
            let mut best: Option<(usize, &C)> = None;
            for (index, cost) in row.iter().enumerate() {
                let Some(cost) = cost else {
                    continue;
                };
                let replace = match best {
                    Some((_, current)) => {
                        compare(cost, current, "alignment endpoint ordering")? == Ordering::Less
                    }
                    None => true,
                };
                if replace {
                    best = Some((index, cost));
                }
            }
            best.map(|(index, _)| index).ok_or(GraphError::Disconnected)
        }
    }
}

fn reconstruct<C: FiniteCost>(
    cells: &Table<C>,
    mut i: usize,
    mut j: usize,
    boundary: AlignmentBoundary,
) -> Result<Vec<AlignmentStep<C>>, GraphError> {
    let mut reversed = Vec::new();
    loop {
        let cell = cells[i][j].as_ref().ok_or_else(|| {
            GraphError::CertificateInvalid("alignment path visits an unreachable cell".to_owned())
        })?;
        let Some(movement) = cell.predecessor else {
            if i == 0 && (j == 0 || boundary == AlignmentBoundary::Subsequence) {
                break;
            }
            return Err(GraphError::CertificateInvalid(
                "alignment path ends at an illegal free boundary".to_owned(),
            ));
        };
        let cost = cell.step_cost.clone().ok_or_else(|| {
            GraphError::CertificateInvalid("alignment step has no cost".to_owned())
        })?;
        match movement {
            AlignmentMove::Match => {
                i -= 1;
                j -= 1;
                reversed.push(AlignmentStep::Match {
                    left: i,
                    right: j,
                    cost,
                });
            }
            AlignmentMove::Delete => {
                i -= 1;
                reversed.push(AlignmentStep::Delete { left: i, cost });
            }
            AlignmentMove::Insert => {
                j -= 1;
                reversed.push(AlignmentStep::Insert { right: j, cost });
            }
        }
    }
    reversed.reverse();
    Ok(reversed)
}

fn validate_policy<C: FiniteCost>(policy: &DtwPolicy<C>) -> Result<(), GraphError> {
    validate_non_negative(&policy.gaps.delete, "alignment deletion gap")?;
    validate_non_negative(&policy.gaps.insert, "alignment insertion gap")
}

fn validate_non_negative<C: FiniteCost>(cost: &C, context: &str) -> Result<(), GraphError> {
    validate(cost, context)?;
    if compare(cost, &C::zero(), context)? == Ordering::Less {
        return Err(GraphError::Unsupported(format!(
            "{context} must be non-negative"
        )));
    }
    Ok(())
}

fn peak_memory(
    left_len: usize,
    right_len: usize,
    memory: AlignmentMemory,
) -> Result<usize, GraphError> {
    let columns = right_len
        .checked_add(1)
        .ok_or_else(|| GraphError::WeightOverflow("alignment columns".to_owned()))?;
    match memory {
        AlignmentMemory::Full => left_len
            .checked_add(1)
            .and_then(|rows| rows.checked_mul(columns))
            .ok_or_else(|| GraphError::WeightOverflow("alignment table cells".to_owned())),
        AlignmentMemory::RollingScoreOnly => {
            let rows: usize = if left_len == 0 { 1 } else { 2 };
            rows.checked_mul(columns)
                .ok_or_else(|| GraphError::WeightOverflow("alignment rolling rows".to_owned()))
        }
    }
}

fn charge_cell(
    meter: &mut Option<&mut WorkMeter<'_>>,
    stats: &mut Stats,
) -> Result<(), GraphError> {
    if let Some(meter) = meter.as_deref_mut() {
        meter.cell()?;
    }
    stats.cells = stats
        .cells
        .checked_add(1)
        .ok_or_else(|| GraphError::WeightOverflow("alignment cell count".to_owned()))?;
    Ok(())
}

fn charge_edge(
    meter: &mut Option<&mut WorkMeter<'_>>,
    stats: &mut Stats,
) -> Result<(), GraphError> {
    if let Some(meter) = meter.as_deref_mut() {
        meter.edge()?;
    }
    stats.edges = stats
        .edges
        .checked_add(1)
        .ok_or_else(|| GraphError::WeightOverflow("alignment edge count".to_owned()))?;
    Ok(())
}

#[cfg(test)]
mod tests;
