//! Verifiable certificates for MST and shortest-path results.
//!
//! Every certificate-producing algorithm ships a verifier that re-checks the
//! result from scratch and never trusts the producer. `verify_mst` checks edge
//! count, acyclicity, spanning connectivity, the recorded total weight, and
//! optimality (the cycle property). `verify_shortest_paths` checks predecessor
//! consistency and edge relaxation.

use crate::error::GraphError;
use crate::graph::Graph;
use crate::unionfind::UnionFind;
use core::fmt::Display;
use core::ops::Add;

/// A spanning tree result: tree edge ids (ascending) and the total weight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanningTree<W> {
    /// Edge ids forming the tree, sorted ascending.
    pub edges: Vec<usize>,
    /// Sum of the tree edge weights.
    pub total_weight: W,
}

/// A compact, checkable MST witness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MstCertificate {
    /// Edge ids claimed to form the minimum spanning tree.
    pub edge_ids: Vec<usize>,
    /// The claimed total weight, rendered via `Display`.
    pub total_weight_repr: String,
}

impl<W: Display> SpanningTree<W> {
    /// The certificate corresponding to this spanning tree.
    pub fn certificate(&self) -> MstCertificate {
        MstCertificate {
            edge_ids: self.edges.clone(),
            total_weight_repr: format!("{}", self.total_weight),
        }
    }
}

fn invalid(msg: &str) -> GraphError {
    GraphError::CertificateInvalid(msg.to_string())
}

/// Maximum edge weight on the unique tree path from `u` to `v`, or `None` if
/// `u == v` or they are not connected in the tree.
fn tree_path_max<W: Ord + Clone>(
    n: usize,
    tree_adj: &[Vec<(usize, W)>],
    u: usize,
    v: usize,
) -> Option<W> {
    let mut max_w: Vec<Option<W>> = vec![None; n];
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    visited[u] = true;
    queue.push_back(u);
    while let Some(x) = queue.pop_front() {
        if x == v {
            return max_w[x].clone();
        }
        for (y, w) in &tree_adj[x] {
            if !visited[*y] {
                visited[*y] = true;
                let cand = match &max_w[x] {
                    Some(m) if m >= w => m.clone(),
                    _ => w.clone(),
                };
                max_w[*y] = Some(cand);
                queue.push_back(*y);
            }
        }
    }
    None
}

/// Verify that `cert` describes a minimum spanning tree of `graph`.
pub fn verify_mst<N, W>(graph: &Graph<N, W>, cert: &MstCertificate) -> Result<(), GraphError>
where
    W: Ord + Clone + Default + Add<Output = W> + Display,
{
    graph.validate()?;
    let n = graph.node_count();

    // Resolve tree edges; reject unknown ids and self-loops.
    let mut tree_edges = Vec::with_capacity(cert.edge_ids.len());
    for &id in &cert.edge_ids {
        let e = graph
            .edges
            .get(id)
            .ok_or_else(|| invalid("unknown edge id"))?;
        if e.is_self_loop() {
            return Err(invalid("self-loop in tree"));
        }
        tree_edges.push(e);
    }

    // Edge count: a spanning tree of n nodes has exactly n-1 edges (0 for n<=1).
    let expected = n.saturating_sub(1);
    if tree_edges.len() != expected {
        return Err(invalid("wrong edge count for a spanning tree"));
    }

    // Acyclic + spanning via union-find.
    let mut uf = UnionFind::new(n);
    for e in &tree_edges {
        if !uf.union(e.source, e.target) {
            return Err(invalid("tree contains a cycle"));
        }
    }
    if n > 0 {
        let root = uf.find(0);
        for i in 1..n {
            if uf.find(i) != root {
                return Err(invalid("tree does not span the graph"));
            }
        }
    }

    // Recorded total weight must match the recomputed sum.
    let mut total = W::default();
    for e in &tree_edges {
        total = total + e.weight.clone();
    }
    if format!("{total}") != cert.total_weight_repr {
        return Err(invalid("total weight mismatch"));
    }

    // Optimality (cycle property): no non-tree edge may be cheaper than the
    // heaviest edge on the tree path between its endpoints.
    let tree_ids: std::collections::HashSet<usize> = cert.edge_ids.iter().copied().collect();
    let mut tree_adj: Vec<Vec<(usize, W)>> = vec![Vec::new(); n];
    for e in &tree_edges {
        tree_adj[e.source].push((e.target, e.weight.clone()));
        tree_adj[e.target].push((e.source, e.weight.clone()));
    }
    for e in &graph.edges {
        if tree_ids.contains(&e.id) || e.is_self_loop() {
            continue;
        }
        if let Some(path_max) = tree_path_max(n, &tree_adj, e.source, e.target)
            && e.weight < path_max
        {
            return Err(invalid("not minimal: a cheaper spanning tree exists"));
        }
    }
    Ok(())
}

/// A shortest-path tree witness from a single source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortestPathCertificate {
    /// The source node.
    pub source: usize,
    /// `predecessors[v]` is the node `v` was reached from (`None` for the source
    /// and for unreachable nodes).
    pub predecessors: Vec<Option<usize>>,
}

/// The minimum weight of a `from -> to` edge, respecting directedness.
fn edge_weight<N>(graph: &Graph<N, i64>, from: usize, to: usize) -> Option<i64> {
    let undirected = !graph.is_directed();
    graph
        .edges
        .iter()
        .filter(|e| {
            (e.source == from && e.target == to)
                || (undirected && e.source == to && e.target == from)
        })
        .map(|e| e.weight)
        .min()
}

/// Recursively resolve the tree distance of `v` (memoized). `Ok(None)` means the
/// node is not in the tree (no predecessor chain to the source).
fn tree_dist<N>(
    graph: &Graph<N, i64>,
    cert: &ShortestPathCertificate,
    v: usize,
    memo: &mut [Option<Option<i64>>],
    visiting: &mut [bool],
) -> Result<Option<i64>, GraphError> {
    if v == cert.source {
        return Ok(Some(0));
    }
    if let Some(d) = memo[v] {
        return Ok(d);
    }
    let result = match cert.predecessors[v] {
        None => None,
        Some(u) => {
            if visiting[v] {
                return Err(invalid("predecessor cycle"));
            }
            visiting[v] = true;
            let du = tree_dist(graph, cert, u, memo, visiting)?;
            visiting[v] = false;
            match du {
                None => return Err(invalid("predecessor points outside the tree")),
                Some(d_u) => {
                    let w = edge_weight(graph, u, v)
                        .ok_or_else(|| invalid("predecessor edge missing"))?;
                    Some(d_u + w)
                }
            }
        }
    };
    memo[v] = Some(result);
    Ok(result)
}

/// Verify that `cert` is a valid shortest-path tree of `graph` from its source.
pub fn verify_shortest_paths<N>(
    graph: &Graph<N, i64>,
    cert: &ShortestPathCertificate,
) -> Result<(), GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    if cert.source >= n {
        return Err(invalid("source out of range"));
    }
    if cert.predecessors.len() != n {
        return Err(invalid("predecessor length mismatch"));
    }
    if cert.predecessors[cert.source].is_some() {
        return Err(invalid("source must have no predecessor"));
    }

    let mut memo: Vec<Option<Option<i64>>> = vec![None; n];
    let mut visiting = vec![false; n];
    let mut dist = vec![None; n];
    for (v, slot) in dist.iter_mut().enumerate() {
        *slot = tree_dist(graph, cert, v, &mut memo, &mut visiting)?;
    }

    // Relaxation + completeness: every edge reachable from the source must keep
    // dist[target] <= dist[source] + weight, and its target must be in the tree.
    let undirected = !graph.is_directed();
    for e in &graph.edges {
        let mut arcs = vec![(e.source, e.target)];
        if undirected {
            arcs.push((e.target, e.source));
        }
        for (a, b) in arcs {
            if let Some(da) = dist[a] {
                match dist[b] {
                    None => return Err(invalid("reachable node missing from tree")),
                    Some(db) => {
                        if db > da + e.weight {
                            return Err(invalid("edge violates shortest-path relaxation"));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::Directedness;
    use crate::path::bellman_ford;

    fn weighted() -> Graph<u8, i64> {
        let mut g = Graph::with_nodes(vec![0, 1, 2, 3], Directedness::Directed);
        g.add_edge(0, 1, 1).unwrap();
        g.add_edge(1, 2, 2).unwrap();
        g.add_edge(0, 2, 5).unwrap();
        g.add_edge(2, 3, 1).unwrap();
        g
    }

    #[test]
    fn valid_shortest_path_cert_verifies() {
        let g = weighted();
        let (res, _) = bellman_ford(&g, 0).unwrap();
        let cert = ShortestPathCertificate {
            source: 0,
            predecessors: res.predecessors,
        };
        assert!(verify_shortest_paths(&g, &cert).is_ok());
    }

    #[test]
    fn tampered_shortest_path_cert_rejected() {
        let g = weighted();
        let (res, _) = bellman_ford(&g, 0).unwrap();
        let mut preds = res.predecessors;
        // Claim node 2 was reached directly from 0 (dist 5) -- but 0->1->2 (3)
        // is shorter, so relaxation of edge 1->2 must fail.
        preds[2] = Some(0);
        let cert = ShortestPathCertificate {
            source: 0,
            predecessors: preds,
        };
        assert!(verify_shortest_paths(&g, &cert).is_err());
    }
}
