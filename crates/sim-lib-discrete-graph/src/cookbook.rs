//! Deterministic cookbook builders for discrete graph recipes.

// conformance: staged Bellman selection composes with verified DTW without a copied DP loop.

use crate::{
    Directedness, DtwPolicy, GapPolicy, Graph, GraphError, bfs, dynamic_time_warp, kruskals_mst,
    layered_shortest_path, verify_alignment, verify_layered_path,
};

/// Report produced by the tiny graph cookbook recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TinyGraphDemo {
    /// Number of graph nodes.
    pub node_count: usize,
    /// Number of graph edges.
    pub edge_count: usize,
    /// Breadth-first traversal order from node 0.
    pub bfs_order: Vec<usize>,
    /// Minimum-spanning-tree edge ids.
    pub mst_edge_ids: Vec<usize>,
    /// Minimum-spanning-tree total weight.
    pub mst_total_weight: i64,
}

/// Joint evidence from composing staged dynamic programming and DTW alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlignmentCompositionDemo {
    /// States selected by certified staged optimization.
    pub staged_states: Vec<i64>,
    /// Stable state index selected in every stage.
    pub staged_indices: Vec<usize>,
    /// Exact staged transition cost.
    pub staged_cost: i64,
    /// Bellman cells retained by staged optimization.
    pub staged_cells: u64,
    /// Staged transitions examined.
    pub staged_edges: u64,
    /// Exact DTW/edit alignment score.
    pub alignment_score: i64,
    /// Number of stable operations in the retained alignment path.
    pub alignment_steps: usize,
    /// Prefix-grid cells retained by full-memory DTW.
    pub alignment_cells: u64,
    /// DTW predecessor edges examined.
    pub alignment_edges: u64,
}

/// Build the modeled graph traversal and MST report used by the cookbook.
pub fn tiny_graph_demo() -> Result<TinyGraphDemo, GraphError> {
    let mut graph = Graph::with_nodes(vec![0, 1, 2], Directedness::Undirected);
    graph.add_edge(0, 1, 1_i64)?;
    graph.add_edge(1, 2, 2_i64)?;
    graph.add_edge(0, 2, 5_i64)?;

    let traversal = bfs(&graph, 0)?;
    let mst = kruskals_mst(&graph)?;

    Ok(TinyGraphDemo {
        node_count: graph.node_count(),
        edge_count: graph.edge_count(),
        bfs_order: traversal.order,
        mst_edge_ids: mst.edges,
        mst_total_weight: mst.total_weight,
    })
}

/// Runs the reusable staged-path and DTW owners as one checked composition.
///
/// This is the adapter point for statistics, music, or analysis callers that
/// first select states and then align sequences. It deliberately invokes both
/// public algorithms and both verifiers rather than carrying another DP loop.
pub fn alignment_composition_demo() -> Result<AlignmentCompositionDemo, GraphError> {
    let layers = vec![vec![0_i64, 3], vec![2_i64, 5], vec![4_i64, 7]];
    let staged = layered_shortest_path(&layers, |left, right| left.abs_diff(*right) as i64)?;
    verify_layered_path(
        &layers,
        |left, right| Some(left.abs_diff(*right) as i64),
        &staged,
    )?;

    let left = [0_i64, 2, 4];
    let right = [0_i64, 1, 2, 4];
    let policy = DtwPolicy::new(GapPolicy::new(2_i64, 2_i64));
    let alignment = dynamic_time_warp(
        &left,
        &right,
        |left, right| left.abs_diff(*right) as i64,
        policy.clone(),
    )?;
    verify_alignment(
        &left,
        &right,
        |left, right| left.abs_diff(*right) as i64,
        &policy,
        &alignment,
    )?;

    Ok(AlignmentCompositionDemo {
        staged_states: staged.states,
        staged_indices: staged.indices,
        staged_cost: staged.total_cost,
        staged_cells: staged.receipt.cells,
        staged_edges: staged.receipt.edges,
        alignment_score: alignment.score,
        alignment_steps: alignment.steps.as_ref().map_or(0, Vec::len),
        alignment_cells: alignment.receipt.cells,
        alignment_edges: alignment.receipt.edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_demo_runs_bfs_and_mst() {
        let demo = tiny_graph_demo().expect("valid graph demo");

        assert_eq!(demo.node_count, 3);
        assert_eq!(demo.edge_count, 3);
        assert_eq!(demo.bfs_order, vec![0, 1, 2]);
        assert_eq!(demo.mst_edge_ids, vec![0, 1]);
        assert_eq!(demo.mst_total_weight, 3);
    }

    #[test]
    fn staged_selection_composes_with_dtw_and_verifies_both_certificates() {
        let demo = alignment_composition_demo().expect("composed alignment evidence");
        assert_eq!(demo.staged_states, vec![3, 2, 4]);
        assert_eq!(demo.staged_indices, vec![1, 0, 0]);
        assert_eq!(demo.staged_cost, 3);
        assert_eq!(demo.alignment_score, 2);
        assert_eq!(demo.alignment_steps, 4);
        assert!(demo.staged_cells > 0 && demo.staged_edges > 0);
        assert!(demo.alignment_cells > 0 && demo.alignment_edges > 0);
    }
}
