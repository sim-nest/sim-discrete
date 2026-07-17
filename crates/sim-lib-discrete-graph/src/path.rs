//! Shortest paths: single-source Dijkstra and Bellman-Ford, plus all-pairs and
//! reachability as thin wrappers over the algebra spine's semiring closure.

use crate::error::GraphError;
use crate::graph::Graph;
use core::cmp::Reverse;
use sim_lib_discrete_algebra::{AlgebraLimits, BoolRing, Matrix, MinPlus};
use std::collections::BinaryHeap;

/// Single-source distances and predecessor forest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathResult<W> {
    /// `distances[v]` is the shortest distance to `v`, or `None` if unreachable.
    pub distances: Vec<Option<W>>,
    /// `predecessors[v]` is the node `v` was reached from on a shortest path.
    pub predecessors: Vec<Option<usize>>,
}

/// Directed out-arcs `(target, weight)` of `node`, honoring directedness.
fn out_arcs<N, W: Clone>(graph: &Graph<N, W>, node: usize) -> Vec<(usize, W)> {
    let undirected = !graph.is_directed();
    let mut arcs = Vec::new();
    for e in &graph.edges {
        if e.source == node {
            arcs.push((e.target, e.weight.clone()));
        } else if undirected && e.target == node {
            arcs.push((e.source, e.weight.clone()));
        }
    }
    arcs
}

/// Dijkstra's algorithm over non-negative `u64` weights.
///
/// # Examples
///
/// On a directed graph where the two-hop route `0 -> 1 -> 2` (1 + 2 = 3) beats
/// the direct edge `0 -> 2` (5), the shortest distance to node `2` is `3`:
///
/// ```
/// use sim_lib_discrete_graph::{dijkstra, Directedness, Graph};
///
/// let mut g: Graph<(), u64> = Graph::with_nodes(vec![(), (), ()], Directedness::Directed);
/// g.add_edge(0, 1, 1).unwrap();
/// g.add_edge(1, 2, 2).unwrap();
/// g.add_edge(0, 2, 5).unwrap();
///
/// let r = dijkstra(&g, 0).unwrap();
/// assert_eq!(r.distances, vec![Some(0), Some(1), Some(3)]);
/// assert_eq!(r.predecessors[2], Some(1)); // reached via node 1
/// ```
pub fn dijkstra<N>(graph: &Graph<N, u64>, source: usize) -> Result<PathResult<u64>, GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    if source >= n {
        return Err(GraphError::NodeOutOfRange {
            node: source,
            count: n,
        });
    }
    let mut dist = vec![None; n];
    let mut pred = vec![None; n];
    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();
    dist[source] = Some(0);
    heap.push(Reverse((0, source)));
    while let Some(Reverse((d, u))) = heap.pop() {
        if dist[u].is_some_and(|best| d > best) {
            continue;
        }
        for (v, w) in out_arcs(graph, u) {
            // Saturating adversarial weights must not wrap into a spuriously
            // short distance; an overflowing relaxation is simply no shorter path.
            let Some(nd) = d.checked_add(w) else {
                continue;
            };
            if dist[v].is_none_or(|best| nd < best) {
                dist[v] = Some(nd);
                pred[v] = Some(u);
                heap.push(Reverse((nd, v)));
            }
        }
    }
    Ok(PathResult {
        distances: dist,
        predecessors: pred,
    })
}

/// Bellman-Ford over `i64` weights. Returns the result and whether a
/// negative-weight cycle is reachable from the source.
pub fn bellman_ford<N>(
    graph: &Graph<N, i64>,
    source: usize,
) -> Result<(PathResult<i64>, bool), GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    if source >= n {
        return Err(GraphError::NodeOutOfRange {
            node: source,
            count: n,
        });
    }
    let undirected = !graph.is_directed();
    // Collect all directed arcs once.
    let mut arcs: Vec<(usize, usize, i64)> = Vec::new();
    for e in &graph.edges {
        arcs.push((e.source, e.target, e.weight));
        if undirected {
            arcs.push((e.target, e.source, e.weight));
        }
    }
    let mut dist: Vec<Option<i64>> = vec![None; n];
    let mut pred = vec![None; n];
    dist[source] = Some(0);
    for _ in 0..n.saturating_sub(1) {
        let mut changed = false;
        for &(a, b, w) in &arcs {
            if let Some(da) = dist[a] {
                let nd = da.checked_add(w).ok_or_else(|| {
                    GraphError::WeightOverflow("Bellman-Ford relaxation".to_string())
                })?;
                if dist[b].is_none_or(|best| nd < best) {
                    dist[b] = Some(nd);
                    pred[b] = Some(a);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut negative_cycle = false;
    for &(a, b, w) in &arcs {
        if let Some(da) = dist[a] {
            let nd = da.checked_add(w).ok_or_else(|| {
                GraphError::WeightOverflow("Bellman-Ford cycle check".to_string())
            })?;
            if dist[b].is_none_or(|best| nd < best) {
                negative_cycle = true;
                break;
            }
        }
    }
    Ok((
        PathResult {
            distances: dist,
            predecessors: pred,
        },
        negative_cycle,
    ))
}

/// All-pairs shortest paths as the min-plus closure of the adjacency matrix.
/// This is a thin wrapper over the spine; it does not re-implement Floyd-Warshall.
pub fn all_pairs_shortest_paths<N>(graph: &Graph<N, i64>) -> Result<Matrix<MinPlus>, GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    let undirected = !graph.is_directed();
    let mut m = Matrix::filled(n, n, MinPlus::Inf);
    for e in &graph.edges {
        // Accumulate by semiring add (min) so parallel edges keep the cheapest.
        m.data[e.source * n + e.target] = min_plus_add(m.data[e.source * n + e.target], e.weight);
        if undirected {
            m.data[e.target * n + e.source] =
                min_plus_add(m.data[e.target * n + e.source], e.weight);
        }
    }
    Ok(m.closure(AlgebraLimits::default())?)
}

fn min_plus_add(cur: MinPlus, w: i64) -> MinPlus {
    use sim_lib_discrete_algebra::Semiring;
    cur.add(&MinPlus::Fin(w))
}

/// Reachability as the boolean closure of the adjacency matrix. Thin wrapper.
pub fn reachability<N, W>(graph: &Graph<N, W>) -> Result<Matrix<BoolRing>, GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    let undirected = !graph.is_directed();
    let mut m = Matrix::filled(n, n, BoolRing(false));
    for e in &graph.edges {
        m.data[e.source * n + e.target] = BoolRing(true);
        if undirected {
            m.data[e.target * n + e.source] = BoolRing(true);
        }
    }
    Ok(m.closure(AlgebraLimits::default())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::Directedness;

    #[test]
    fn dijkstra_row_equals_all_pairs_row() {
        // Same structure over u64 (Dijkstra) and i64 (all-pairs closure).
        let edges = [(0usize, 1usize, 1u64), (1, 2, 2), (0, 2, 5), (2, 3, 1)];
        let mut gu: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2, 3], Directedness::Directed);
        let mut gi: Graph<u8, i64> = Graph::with_nodes(vec![0, 1, 2, 3], Directedness::Directed);
        for &(s, t, w) in &edges {
            gu.add_edge(s, t, w).unwrap();
            gi.add_edge(s, t, w as i64).unwrap();
        }
        let dj = dijkstra(&gu, 0).unwrap();
        let ap = all_pairs_shortest_paths(&gi).unwrap();
        for j in 0..4 {
            let from_closure = match ap.data[j] {
                MinPlus::Fin(d) => Some(d as u64),
                MinPlus::Inf => None,
            };
            assert_eq!(dj.distances[j], from_closure, "node {j}");
        }
    }

    #[test]
    fn bellman_ford_handles_negative_edge_without_cycle() {
        let mut g: Graph<u8, i64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Directed);
        g.add_edge(0, 1, 4).unwrap();
        g.add_edge(0, 2, 5).unwrap();
        g.add_edge(2, 1, -3).unwrap(); // 0->2->1 = 2 beats direct 4
        let (res, neg) = bellman_ford(&g, 0).unwrap();
        assert!(!neg);
        assert_eq!(res.distances[1], Some(2));
    }

    #[test]
    fn bellman_ford_detects_negative_cycle() {
        let mut g: Graph<u8, i64> = Graph::with_nodes(vec![0, 1], Directedness::Directed);
        g.add_edge(0, 1, 1).unwrap();
        g.add_edge(1, 0, -2).unwrap(); // cycle weight -1
        let (_res, neg) = bellman_ford(&g, 0).unwrap();
        assert!(neg);
    }

    #[test]
    fn near_max_weights_do_not_wrap_distance() {
        // Dijkstra: two near-u64::MAX hops must not wrap to a tiny distance.
        let mut gu: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Directed);
        gu.add_edge(0, 1, u64::MAX - 1).unwrap();
        gu.add_edge(1, 2, u64::MAX - 1).unwrap();
        let dj = dijkstra(&gu, 0).unwrap();
        assert_eq!(dj.distances[1], Some(u64::MAX - 1));
        // 2 is only reachable via an overflowing relaxation, so it stays unreached.
        assert_eq!(dj.distances[2], None);

        // Bellman-Ford: two near-i64::MAX hops fail closed instead of wrapping
        // or silently dropping the overflowing reachable relaxation.
        let mut gi: Graph<u8, i64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Directed);
        gi.add_edge(0, 1, i64::MAX - 1).unwrap();
        gi.add_edge(1, 2, i64::MAX - 1).unwrap();
        assert!(matches!(
            bellman_ford(&gi, 0),
            Err(GraphError::WeightOverflow(_))
        ));
    }

    #[test]
    fn bellman_ford_rejects_negative_overflow() {
        let mut g: Graph<u8, i64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Directed);
        g.add_edge(0, 1, i64::MIN + 1).unwrap();
        g.add_edge(1, 2, -2).unwrap();

        assert!(matches!(
            bellman_ford(&g, 0),
            Err(GraphError::WeightOverflow(_))
        ));
    }

    #[test]
    fn reachability_is_transitive() {
        let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Directed);
        g.add_edge(0, 1, 1).unwrap();
        g.add_edge(1, 2, 1).unwrap();
        let r = reachability(&g).unwrap();
        assert_eq!(r.data[2], BoolRing(true)); // 0 reaches 2
        assert_eq!(r.data[6], BoolRing(false)); // 2 does not reach 0
    }
}
