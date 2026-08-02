use super::{
    Alignment, AlignmentCertificate, AlignmentMemory, DtwPolicy, full_table, peak_memory,
    reconstruct, rolling_row, select_endpoint, select_rolling_endpoint, validate_policy,
};
use crate::{FiniteCost, GraphError};

/// Verifies an alignment certificate, deterministic ties, path, and receipt.
pub fn verify_alignment<T, C: FiniteCost>(
    left: &[T],
    right: &[T],
    local_cost: impl Fn(&T, &T) -> C,
    policy: &DtwPolicy<C>,
    alignment: &Alignment<C>,
) -> Result<(), GraphError> {
    validate_policy(policy)?;
    alignment.receipt.validate()?;
    let (score, steps, certificate, stats) = match policy.memory {
        AlignmentMemory::Full => {
            let (cells, stats) = full_table(left, right, &local_cost, policy, None)?;
            let endpoint = select_endpoint(&cells, policy.boundary)?;
            let score = cells[left.len()][endpoint]
                .as_ref()
                .expect("selected endpoint is reachable")
                .total_cost
                .clone();
            let steps = reconstruct(&cells, left.len(), endpoint, policy.boundary)?;
            (
                score,
                Some(steps),
                AlignmentCertificate::Full { cells },
                stats,
            )
        }
        AlignmentMemory::RollingScoreOnly => {
            let (final_row, stats) = rolling_row(left, right, &local_cost, policy, None)?;
            let endpoint = select_rolling_endpoint(&final_row, policy.boundary, right.len())?;
            let score = final_row[endpoint]
                .as_ref()
                .expect("selected endpoint is reachable")
                .clone();
            (
                score,
                None,
                AlignmentCertificate::Rolling {
                    final_row,
                    endpoint,
                },
                stats,
            )
        }
    };
    if alignment.score != score || alignment.steps != steps || alignment.certificate != certificate
    {
        return Err(GraphError::CertificateInvalid(
            "alignment result violates its Bellman recurrence or stable ties".to_owned(),
        ));
    }
    let memory = peak_memory(left.len(), right.len(), policy.memory)?;
    if alignment.receipt.cells != stats.cells
        || alignment.receipt.edges != stats.edges
        || alignment.receipt.peak_memory_cells != memory
    {
        return Err(GraphError::CertificateInvalid(
            "alignment receipt does not match evaluated cells and edges".to_owned(),
        ));
    }
    Ok(())
}
