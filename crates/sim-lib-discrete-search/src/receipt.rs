//! Search receipts and run summaries.

use crate::SearchControl;

/// Completion status recorded by a bounded search run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchStatus {
    /// The frontier exhausted after emitting at least one output.
    Complete,
    /// The run stopped at a configured bound and may have partial outputs.
    Partial,
    /// The interrupt source cancelled the run.
    Cancelled,
    /// The frontier exhausted without any output.
    Infeasible,
}

/// Receipt describing how a bounded search run ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchReceipt {
    /// Final run status.
    pub status: SearchStatus,
    /// Stable reason for non-complete statuses.
    pub reason: Option<String>,
    /// Total charged work.
    pub work_used: u64,
    /// Number of states expanded.
    pub expanded: u64,
    /// Number of states scored.
    pub scored: u64,
    /// Number of states propagated.
    pub propagated: u64,
    /// Number of outputs emitted.
    pub emitted: u64,
    /// Number of locally pruned prefixes or beam drops.
    pub pruned: u64,
    /// Largest observed frontier length.
    pub max_frontier: usize,
    /// Number of outputs returned to the caller.
    pub result_count: usize,
    /// Caller-supplied deterministic seed.
    pub seed: u64,
    /// Stable digest of the control policy.
    pub policy_digest: String,
    /// Stable digest of the policy, metrics, status, and output order.
    pub digest: String,
}

impl SearchReceipt {
    pub(crate) fn invalid(control: &SearchControl, reason: String) -> Self {
        Self::finalize(
            SearchStatus::Partial,
            Some(format!("invalid control: {reason}")),
            Metrics::default(),
            control,
            &[],
        )
    }

    pub(crate) fn finalize(
        status: SearchStatus,
        reason: Option<String>,
        metrics: Metrics,
        control: &SearchControl,
        output_material: &[String],
    ) -> Self {
        let policy_material = control.policy_material();
        let policy_digest = stable_digest(&[policy_material.as_str()]);
        let status_material = format!("{status:?}");
        let reason_material = reason.clone().unwrap_or_default();
        let metrics_material = format!(
            "work={};expanded={};scored={};propagated={};emitted={};pruned={};max_frontier={};result_count={}",
            metrics.work_used,
            metrics.expanded,
            metrics.scored,
            metrics.propagated,
            metrics.emitted,
            metrics.pruned,
            metrics.max_frontier,
            metrics.result_count,
        );
        let mut parts = vec![
            policy_material.as_str(),
            status_material.as_str(),
            reason_material.as_str(),
            metrics_material.as_str(),
        ];
        parts.extend(output_material.iter().map(String::as_str));
        let digest = stable_digest(&parts);

        Self {
            status,
            reason,
            work_used: metrics.work_used,
            expanded: metrics.expanded,
            scored: metrics.scored,
            propagated: metrics.propagated,
            emitted: metrics.emitted,
            pruned: metrics.pruned,
            max_frontier: metrics.max_frontier,
            result_count: metrics.result_count,
            seed: control.seed,
            policy_digest,
            digest,
        }
    }
}

/// Outputs and receipt produced by one search run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRun<Output> {
    /// Outputs emitted before the run stopped.
    pub outputs: Vec<Output>,
    /// Receipt covering status, bounds, work, and deterministic digests.
    pub receipt: SearchReceipt,
}

/// Mutable counters used while solving.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Metrics {
    pub(crate) work_used: u64,
    pub(crate) expanded: u64,
    pub(crate) scored: u64,
    pub(crate) propagated: u64,
    pub(crate) emitted: u64,
    pub(crate) pruned: u64,
    pub(crate) max_frontier: usize,
    pub(crate) result_count: usize,
}

pub(crate) fn stable_digest(parts: &[&str]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
