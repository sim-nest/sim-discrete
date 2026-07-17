//! The weighted graph value type and its adjacency expansion.

use crate::edge::{Directedness, Edge};
use crate::error::GraphError;

/// A weighted graph over node labels `N` and edge weights `W`.
///
/// Algorithm-level node identity is the `usize` index into `nodes`; labels are
/// payload and never affect correctness. Edge ids are stable. Multiedges and
/// self-loops are representable. Undirected graphs store one record per edge;
/// [`Graph::neighbors`] expands both directions.
///
/// # Examples
///
/// Build a small directed graph and read a node's outgoing adjacencies:
///
/// ```
/// use sim_lib_discrete_graph::{Directedness, Graph};
///
/// let mut g: Graph<&str, u64> = Graph::new(Directedness::Directed);
/// let a = g.add_node("a");
/// let b = g.add_node("b");
/// let c = g.add_node("c");
/// g.add_edge(a, b, 1).unwrap();
/// g.add_edge(a, c, 2).unwrap();
///
/// assert_eq!(g.node_count(), 3);
/// assert_eq!(g.edge_count(), 2);
///
/// let neighbors: Vec<usize> = g.neighbors(a).unwrap().iter().map(|n| n.node).collect();
/// assert_eq!(neighbors, vec![b, c]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Graph<N, W> {
    /// Node labels, indexed by node id.
    pub nodes: Vec<N>,
    /// Edge records.
    pub edges: Vec<Edge<W>>,
    /// Whether edges are directed.
    pub directedness: Directedness,
}

/// One adjacency: the edge taken and the neighbor reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Neighbor<'a, W> {
    /// The traversed edge id.
    pub edge_id: usize,
    /// The neighbor node index.
    pub node: usize,
    /// The traversed edge weight.
    pub weight: &'a W,
}

impl<N, W> Graph<N, W> {
    /// An empty graph with the given directedness.
    pub fn new(directedness: Directedness) -> Self {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
            directedness,
        }
    }

    /// A graph seeded with `nodes` and no edges.
    pub fn with_nodes(nodes: Vec<N>, directedness: Directedness) -> Self {
        Graph {
            nodes,
            edges: Vec::new(),
            directedness,
        }
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edge records (one per undirected edge).
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Whether the graph is directed.
    pub fn is_directed(&self) -> bool {
        matches!(self.directedness, Directedness::Directed)
    }

    /// Append a node label, returning its index.
    pub fn add_node(&mut self, label: N) -> usize {
        self.nodes.push(label);
        self.nodes.len() - 1
    }

    /// Append an edge, validating endpoints and assigning a fresh id.
    pub fn add_edge(
        &mut self,
        source: usize,
        target: usize,
        weight: W,
    ) -> Result<usize, GraphError> {
        let n = self.nodes.len();
        let id = self.edges.len();
        for node in [source, target] {
            if node >= n {
                return Err(GraphError::InvalidEndpoint {
                    edge: id,
                    node,
                    len: n,
                });
            }
        }
        self.edges.push(Edge {
            id,
            source,
            target,
            weight,
        });
        Ok(id)
    }

    /// Validate that edge ids match storage order and every endpoint is in range.
    ///
    /// Public fields keep the value easy to encode, but id-indexed consumers
    /// require `edges[i].id == i`. Sparse, duplicate, or shuffled ids are
    /// rejected before bridge and certificate code indexes by edge id.
    pub fn validate(&self) -> Result<(), GraphError> {
        let n = self.nodes.len();
        let edge_count = self.edges.len();
        for (index, e) in self.edges.iter().enumerate() {
            if e.id != index {
                return Err(GraphError::InvalidEdgeId {
                    index,
                    id: e.id,
                    len: edge_count,
                });
            }
            for node in [e.source, e.target] {
                if node >= n {
                    return Err(GraphError::InvalidEndpoint {
                        edge: e.id,
                        node,
                        len: n,
                    });
                }
            }
        }
        Ok(())
    }

    /// Outgoing adjacencies of `node`, in deterministic ascending neighbor order
    /// (ties broken by edge id). For undirected graphs both directions expand.
    pub fn neighbors(&self, node: usize) -> Result<Vec<Neighbor<'_, W>>, GraphError> {
        if node >= self.nodes.len() {
            return Err(GraphError::NodeOutOfRange {
                node,
                count: self.nodes.len(),
            });
        }
        let directed = self.is_directed();
        let mut out = Vec::new();
        for e in &self.edges {
            if e.source == node {
                out.push(Neighbor {
                    edge_id: e.id,
                    node: e.target,
                    weight: &e.weight,
                });
            } else if !directed && e.target == node {
                out.push(Neighbor {
                    edge_id: e.id,
                    node: e.source,
                    weight: &e.weight,
                });
            }
        }
        out.sort_by_key(|adj| (adj.node, adj.edge_id));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_is_valid() {
        let g: Graph<(), ()> = Graph::new(Directedness::Undirected);
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
        assert!(g.validate().is_ok());
    }

    #[test]
    fn add_edge_rejects_invalid_endpoint() {
        let mut g: Graph<&str, u64> = Graph::with_nodes(vec!["a", "b"], Directedness::Directed);
        let r = g.add_edge(0, 5, 1);
        assert!(matches!(
            r,
            Err(GraphError::InvalidEndpoint { node: 5, .. })
        ));
    }

    #[test]
    fn validate_rejects_sparse_edge_id() {
        let g = Graph {
            nodes: vec![0, 1],
            edges: vec![Edge {
                id: 2,
                source: 0,
                target: 1,
                weight: 7,
            }],
            directedness: Directedness::Directed,
        };

        assert!(matches!(
            g.validate(),
            Err(GraphError::InvalidEdgeId {
                index: 0,
                id: 2,
                len: 1,
            })
        ));
    }

    #[test]
    fn validate_rejects_duplicate_edge_id() {
        let g = Graph {
            nodes: vec![0, 1, 2],
            edges: vec![
                Edge {
                    id: 0,
                    source: 0,
                    target: 1,
                    weight: 7,
                },
                Edge {
                    id: 0,
                    source: 1,
                    target: 2,
                    weight: 9,
                },
            ],
            directedness: Directedness::Directed,
        };

        assert!(matches!(
            g.validate(),
            Err(GraphError::InvalidEdgeId {
                index: 1,
                id: 0,
                len: 2,
            })
        ));
    }

    #[test]
    fn self_loop_and_multiedge_preserved() {
        let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1], Directedness::Undirected);
        g.add_edge(0, 0, 1).unwrap(); // self-loop
        g.add_edge(0, 1, 2).unwrap();
        g.add_edge(0, 1, 3).unwrap(); // parallel edge
        assert_eq!(g.edge_count(), 3);
        assert!(g.edges[0].is_self_loop());
    }

    #[test]
    fn undirected_neighbors_expand_both_ways() {
        let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Undirected);
        g.add_edge(0, 1, 10).unwrap();
        g.add_edge(2, 1, 20).unwrap();
        let n1: Vec<usize> = g.neighbors(1).unwrap().iter().map(|a| a.node).collect();
        assert_eq!(n1, vec![0, 2]); // sorted ascending
    }

    #[test]
    fn directed_neighbors_are_outgoing_only() {
        let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1], Directedness::Directed);
        g.add_edge(0, 1, 1).unwrap();
        assert_eq!(g.neighbors(0).unwrap().len(), 1);
        assert_eq!(g.neighbors(1).unwrap().len(), 0);
    }
}
