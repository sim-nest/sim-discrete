//! Edge records and graph directedness.

/// Whether a graph's edges are directed or undirected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directedness {
    /// Edges are one-way (`source -> target`).
    Directed,
    /// Edges are two-way; one record stands for both directions.
    Undirected,
}

/// A weighted edge. `id` is stable within a graph; `source`/`target` are node
/// indices into the graph's `nodes`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge<W> {
    /// Stable edge id within its graph.
    pub id: usize,
    /// Source node index.
    pub source: usize,
    /// Target node index.
    pub target: usize,
    /// Edge weight / payload.
    pub weight: W,
}

impl<W> Edge<W> {
    /// Whether this edge is a self-loop (`source == target`).
    pub fn is_self_loop(&self) -> bool {
        self.source == self.target
    }

    /// The endpoint opposite `node`, or `None` if `node` is not an endpoint.
    /// For a self-loop, returns `node` when `node` is the endpoint.
    pub fn other(&self, node: usize) -> Option<usize> {
        if self.source == node {
            Some(self.target)
        } else if self.target == node {
            Some(self.source)
        } else {
            None
        }
    }
}
