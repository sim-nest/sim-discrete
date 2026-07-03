//! `metric/discrete/spanning-tree-swap`: the edge-exchange distance between two
//! spanning trees of the same graph.
//!
//! Two spanning trees of an `n`-node graph each have `n-1` edges, so the number
//! of edge swaps to turn one into the other is `|A \ B|` (equivalently half the
//! symmetric difference). This reuses the same set reasoning the MST verifier
//! uses, rather than a new graph traversal.

use crate::error::RankAdapterError;
use std::collections::BTreeSet;

/// The edge-swap distance between two spanning trees, given as edge-id sets.
/// Both must have the same number of edges.
pub fn spanning_tree_swap_distance(a: &[usize], b: &[usize]) -> Result<usize, RankAdapterError> {
    if a.len() != b.len() {
        return Err(RankAdapterError::Invalid(
            "spanning trees differ in edge count".to_string(),
        ));
    }
    let sb: BTreeSet<usize> = b.iter().copied().collect();
    Ok(a.iter().filter(|id| !sb.contains(id)).count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_trees_have_zero_distance() {
        assert_eq!(
            spanning_tree_swap_distance(&[0, 1, 2], &[0, 1, 2]).unwrap(),
            0
        );
    }

    #[test]
    fn one_edge_swap_is_distance_one() {
        // Trees {0,1,2} and {0,1,3} differ by swapping edge 2 for edge 3.
        assert_eq!(
            spanning_tree_swap_distance(&[0, 1, 2], &[0, 1, 3]).unwrap(),
            1
        );
    }

    #[test]
    fn size_mismatch_rejected() {
        assert!(matches!(
            spanning_tree_swap_distance(&[0, 1], &[0, 1, 2]),
            Err(RankAdapterError::Invalid(_))
        ));
    }
}
