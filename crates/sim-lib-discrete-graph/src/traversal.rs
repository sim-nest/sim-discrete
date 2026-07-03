//! Deterministic breadth-first and depth-first traversal.

use crate::error::GraphError;
use crate::graph::Graph;
use std::collections::VecDeque;

/// The result of a traversal from a single start node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traversal {
    /// Nodes in the order they were first visited.
    pub order: Vec<usize>,
    /// `predecessor[v]` is the node `v` was discovered from (`None` for the
    /// start node and for unreached nodes).
    pub predecessor: Vec<Option<usize>>,
}

fn check_start<N, W>(graph: &Graph<N, W>, start: usize) -> Result<(), GraphError> {
    if start >= graph.node_count() {
        return Err(GraphError::NodeOutOfRange {
            node: start,
            count: graph.node_count(),
        });
    }
    Ok(())
}

/// Breadth-first traversal from `start`. Neighbors are visited in ascending
/// node order, so the result is deterministic.
///
/// # Examples
///
/// On the chain `0 -> 1 -> 2`, BFS from `0` visits the nodes in order and
/// records each node's discovery predecessor:
///
/// ```
/// use sim_lib_discrete_graph::{bfs, Directedness, Graph};
///
/// let mut g: Graph<(), ()> = Graph::with_nodes(vec![(), (), ()], Directedness::Directed);
/// g.add_edge(0, 1, ()).unwrap();
/// g.add_edge(1, 2, ()).unwrap();
///
/// let t = bfs(&g, 0).unwrap();
/// assert_eq!(t.order, vec![0, 1, 2]);
/// assert_eq!(t.predecessor, vec![None, Some(0), Some(1)]);
/// ```
pub fn bfs<N, W>(graph: &Graph<N, W>, start: usize) -> Result<Traversal, GraphError> {
    check_start(graph, start)?;
    let n = graph.node_count();
    let mut visited = vec![false; n];
    let mut predecessor = vec![None; n];
    let mut order = Vec::new();
    let mut queue = VecDeque::new();
    visited[start] = true;
    queue.push_back(start);
    while let Some(u) = queue.pop_front() {
        order.push(u);
        for adj in graph.neighbors(u)? {
            if !visited[adj.node] {
                visited[adj.node] = true;
                predecessor[adj.node] = Some(u);
                queue.push_back(adj.node);
            }
        }
    }
    Ok(Traversal { order, predecessor })
}

/// Depth-first traversal from `start`. Neighbors are explored in ascending node
/// order, so the result is deterministic.
pub fn dfs<N, W>(graph: &Graph<N, W>, start: usize) -> Result<Traversal, GraphError> {
    check_start(graph, start)?;
    let n = graph.node_count();
    let mut visited = vec![false; n];
    let mut predecessor = vec![None; n];
    let mut order = Vec::new();
    // Explicit stack of (node, predecessor). Push neighbors in reverse so the
    // smallest index is popped (and thus visited) first.
    let mut stack = vec![(start, None)];
    while let Some((u, pred)) = stack.pop() {
        if visited[u] {
            continue;
        }
        visited[u] = true;
        predecessor[u] = pred;
        order.push(u);
        let neighbors = graph.neighbors(u)?;
        for adj in neighbors.into_iter().rev() {
            if !visited[adj.node] {
                stack.push((adj.node, Some(u)));
            }
        }
    }
    Ok(Traversal { order, predecessor })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::Directedness;

    fn diamond() -> Graph<u8, u64> {
        // 0 - 1, 0 - 2, 1 - 3, 2 - 3 (undirected diamond).
        let mut g = Graph::with_nodes(vec![0, 1, 2, 3], Directedness::Undirected);
        g.add_edge(0, 1, 1).unwrap();
        g.add_edge(0, 2, 1).unwrap();
        g.add_edge(1, 3, 1).unwrap();
        g.add_edge(2, 3, 1).unwrap();
        g
    }

    #[test]
    fn bfs_is_deterministic() {
        let t = bfs(&diamond(), 0).unwrap();
        assert_eq!(t.order, vec![0, 1, 2, 3]);
        assert_eq!(t.predecessor[3], Some(1)); // reached from the smaller branch
    }

    #[test]
    fn dfs_is_deterministic() {
        let t = dfs(&diamond(), 0).unwrap();
        assert_eq!(t.order, vec![0, 1, 3, 2]);
    }

    #[test]
    fn bad_start_fails() {
        assert!(matches!(
            bfs(&diamond(), 9),
            Err(GraphError::NodeOutOfRange { node: 9, .. })
        ));
    }
}
