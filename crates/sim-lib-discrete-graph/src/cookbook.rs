//! Deterministic cookbook builders for discrete graph recipes.

use crate::{Directedness, Graph, GraphError, bfs, kruskals_mst};

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
}
