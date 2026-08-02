//! Work, memory, deadline, and cancellation control for graph algorithms.

use std::time::{Duration, Instant};

use crate::GraphError;

/// Work charged for one dynamic-programming cell and one examined edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlgorithmWorkCosts {
    /// Work units charged before evaluating a table cell.
    pub cell: u64,
    /// Work units charged before examining a transition or residual edge.
    pub edge: u64,
}

impl Default for AlgorithmWorkCosts {
    fn default() -> Self {
        Self { cell: 1, edge: 1 }
    }
}

/// Bounds and accounting policy for a graph algorithm run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AlgorithmControl {
    /// Optional maximum charged work.
    pub max_work: Option<u64>,
    /// Optional maximum simultaneously retained table cells.
    pub max_memory_cells: Option<usize>,
    /// Optional wall-clock deadline relative to the start of the call.
    pub max_time: Option<Duration>,
    /// Per-operation work costs.
    pub costs: AlgorithmWorkCosts,
}

impl AlgorithmControl {
    /// Returns a copy with a maximum work bound.
    pub fn with_max_work(mut self, max_work: u64) -> Self {
        self.max_work = Some(max_work);
        self
    }

    /// Returns a copy with a maximum retained-cell bound.
    pub fn with_max_memory_cells(mut self, max_memory_cells: usize) -> Self {
        self.max_memory_cells = Some(max_memory_cells);
        self
    }

    /// Returns a copy with a relative wall-clock deadline.
    pub fn with_max_time(mut self, max_time: Duration) -> Self {
        self.max_time = Some(max_time);
        self
    }

    /// Returns a copy with explicit cell and edge charges.
    pub fn with_costs(mut self, costs: AlgorithmWorkCosts) -> Self {
        self.costs = costs;
        self
    }
}

/// Cooperative cancellation source for graph algorithms.
pub trait AlgorithmInterrupt {
    /// Returns true when the active computation should stop.
    fn is_cancelled(&self) -> bool;
}

/// Cancellation source that never interrupts.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeverInterrupt;

impl AlgorithmInterrupt for NeverInterrupt {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Deterministic accounting evidence returned by a completed algorithm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgorithmReceipt {
    /// Total charged work.
    pub work_used: u64,
    /// Number of evaluated dynamic-programming cells.
    pub cells: u64,
    /// Number of examined transitions or residual edges.
    pub edges: u64,
    /// Maximum table cells retained simultaneously.
    pub peak_memory_cells: usize,
    /// Work charged per cell.
    pub cell_work: u64,
    /// Work charged per edge.
    pub edge_work: u64,
}

impl AlgorithmReceipt {
    pub(crate) fn validate(&self) -> Result<(), GraphError> {
        let cell_work = self
            .cells
            .checked_mul(self.cell_work)
            .ok_or_else(|| GraphError::CertificateInvalid("cell work overflowed".to_owned()))?;
        let edge_work = self
            .edges
            .checked_mul(self.edge_work)
            .ok_or_else(|| GraphError::CertificateInvalid("edge work overflowed".to_owned()))?;
        let expected = cell_work
            .checked_add(edge_work)
            .ok_or_else(|| GraphError::CertificateInvalid("total work overflowed".to_owned()))?;
        if self.cell_work == 0 || self.edge_work == 0 {
            return Err(GraphError::CertificateInvalid(
                "algorithm work charges must be positive".to_owned(),
            ));
        }
        if self.work_used != expected {
            return Err(GraphError::CertificateInvalid(
                "algorithm receipt work total is inconsistent".to_owned(),
            ));
        }
        Ok(())
    }
}

pub(crate) struct WorkMeter<'a> {
    control: &'a AlgorithmControl,
    interrupt: &'a dyn AlgorithmInterrupt,
    started: Instant,
    receipt: AlgorithmReceipt,
}

impl<'a> WorkMeter<'a> {
    pub(crate) fn new(
        control: &'a AlgorithmControl,
        interrupt: &'a dyn AlgorithmInterrupt,
        peak_memory_cells: usize,
    ) -> Result<Self, GraphError> {
        if control.costs.cell == 0 || control.costs.edge == 0 {
            return Err(GraphError::InvalidControl(
                "cell and edge work costs must be positive".to_owned(),
            ));
        }
        if control
            .max_memory_cells
            .is_some_and(|limit| peak_memory_cells > limit)
        {
            return Err(GraphError::ControlStopped(format!(
                "memory-cell bound reached: required {peak_memory_cells}"
            )));
        }
        Ok(Self {
            control,
            interrupt,
            started: Instant::now(),
            receipt: AlgorithmReceipt {
                work_used: 0,
                cells: 0,
                edges: 0,
                peak_memory_cells,
                cell_work: control.costs.cell,
                edge_work: control.costs.edge,
            },
        })
    }

    pub(crate) fn cell(&mut self) -> Result<(), GraphError> {
        self.charge(self.control.costs.cell)?;
        self.receipt.cells = self
            .receipt
            .cells
            .checked_add(1)
            .ok_or_else(|| GraphError::WeightOverflow("cell counter".to_owned()))?;
        Ok(())
    }

    pub(crate) fn edge(&mut self) -> Result<(), GraphError> {
        self.charge(self.control.costs.edge)?;
        self.receipt.edges = self
            .receipt
            .edges
            .checked_add(1)
            .ok_or_else(|| GraphError::WeightOverflow("edge counter".to_owned()))?;
        Ok(())
    }

    pub(crate) fn finish(self) -> AlgorithmReceipt {
        self.receipt
    }

    fn charge(&mut self, work: u64) -> Result<(), GraphError> {
        if self.interrupt.is_cancelled() {
            return Err(GraphError::ControlStopped(
                "algorithm interrupt cancelled the run".to_owned(),
            ));
        }
        if self
            .control
            .max_time
            .is_some_and(|limit| self.started.elapsed() >= limit)
        {
            return Err(GraphError::ControlStopped(
                "algorithm time bound reached".to_owned(),
            ));
        }
        let next = self
            .receipt
            .work_used
            .checked_add(work)
            .ok_or_else(|| GraphError::WeightOverflow("work counter".to_owned()))?;
        if self.control.max_work.is_some_and(|limit| next > limit) {
            return Err(GraphError::ControlStopped(
                "algorithm work bound reached".to_owned(),
            ));
        }
        self.receipt.work_used = next;
        Ok(())
    }
}
