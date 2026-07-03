//! Minimum spanning tree via Prim's and Kruskal's algorithms.
//!
//! Both require an undirected graph, ignore self-loops, and break ties
//! deterministically by `(weight, min endpoint, max endpoint, edge id)`. Each
//! returns a [`SpanningTree`] whose edge ids are sorted ascending, so the two
//! algorithms produce comparable witnesses.

use crate::certificate::SpanningTree;
use crate::error::GraphError;
use crate::graph::Graph;
use crate::unionfind::UnionFind;
use core::cmp::Reverse;
use core::ops::Add;
use std::collections::BinaryHeap;

/// Prim heap key: `(weight, min endpoint, max endpoint, edge id, to-node)`.
type PrimHeap<W> = BinaryHeap<Reverse<(W, usize, usize, usize, usize)>>;

fn require_undirected<N, W>(graph: &Graph<N, W>) -> Result<(), GraphError> {
    graph.validate()?;
    if graph.is_directed() {
        return Err(GraphError::WrongGraphKind(
            "MST requires an undirected graph".to_string(),
        ));
    }
    Ok(())
}

/// Kruskal's algorithm: sort edges, union-find to reject cycles.
pub fn kruskals_mst<N, W>(graph: &Graph<N, W>) -> Result<SpanningTree<W>, GraphError>
where
    W: Ord + Clone + Default + Add<Output = W>,
{
    require_undirected(graph)?;
    let n = graph.node_count();
    if n == 0 {
        return Ok(SpanningTree {
            edges: Vec::new(),
            total_weight: W::default(),
        });
    }
    let mut cand: Vec<_> = graph.edges.iter().filter(|e| !e.is_self_loop()).collect();
    cand.sort_by(|a, b| {
        a.weight
            .cmp(&b.weight)
            .then_with(|| a.source.min(a.target).cmp(&b.source.min(b.target)))
            .then_with(|| a.source.max(a.target).cmp(&b.source.max(b.target)))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut uf = UnionFind::new(n);
    let mut chosen = Vec::new();
    let mut total = W::default();
    for e in cand {
        if uf.union(e.source, e.target) {
            chosen.push(e.id);
            total = total + e.weight.clone();
            if chosen.len() == n - 1 {
                break;
            }
        }
    }
    if chosen.len() != n - 1 {
        return Err(GraphError::Disconnected);
    }
    chosen.sort_unstable();
    Ok(SpanningTree {
        edges: chosen,
        total_weight: total,
    })
}

/// Prim's algorithm: grow a tree from node 0 using a min-heap keyed on the same
/// deterministic tie-break as Kruskal.
pub fn prims_mst<N, W>(graph: &Graph<N, W>) -> Result<SpanningTree<W>, GraphError>
where
    W: Ord + Clone + Default + Add<Output = W>,
{
    require_undirected(graph)?;
    let n = graph.node_count();
    if n == 0 {
        return Ok(SpanningTree {
            edges: Vec::new(),
            total_weight: W::default(),
        });
    }
    // adj[x] = (weight, other endpoint, edge id), excluding self-loops.
    let mut adj: Vec<Vec<(W, usize, usize)>> = vec![Vec::new(); n];
    for e in &graph.edges {
        if e.is_self_loop() {
            continue;
        }
        adj[e.source].push((e.weight.clone(), e.target, e.id));
        adj[e.target].push((e.weight.clone(), e.source, e.id));
    }

    let mut in_tree = vec![false; n];
    let mut heap: PrimHeap<W> = BinaryHeap::new();
    let push_incident = |heap: &mut PrimHeap<W>, in_tree: &[bool], x: usize| {
        for (w, other, id) in &adj[x] {
            if !in_tree[*other] {
                heap.push(Reverse((
                    w.clone(),
                    x.min(*other),
                    x.max(*other),
                    *id,
                    *other,
                )));
            }
        }
    };

    in_tree[0] = true;
    push_incident(&mut heap, &in_tree, 0);
    let mut chosen = Vec::new();
    let mut total = W::default();
    let mut count = 1;
    while count < n {
        let Some(Reverse((w, _, _, id, to))) = heap.pop() else {
            break;
        };
        if in_tree[to] {
            continue;
        }
        in_tree[to] = true;
        chosen.push(id);
        total = total + w;
        count += 1;
        push_incident(&mut heap, &in_tree, to);
    }
    if count != n {
        return Err(GraphError::Disconnected);
    }
    chosen.sort_unstable();
    Ok(SpanningTree {
        edges: chosen,
        total_weight: total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::verify_mst;
    use crate::edge::Directedness;

    fn triangle() -> Graph<u8, u64> {
        let mut g = Graph::with_nodes(vec![0, 1, 2], Directedness::Undirected);
        g.add_edge(0, 1, 1).unwrap(); // id 0
        g.add_edge(1, 2, 2).unwrap(); // id 1
        g.add_edge(0, 2, 3).unwrap(); // id 2
        g
    }

    #[test]
    fn triangle_mst_weight_and_edges() {
        let t = kruskals_mst(&triangle()).unwrap();
        assert_eq!(t.total_weight, 3);
        assert_eq!(t.edges, vec![0, 1]);
    }

    #[test]
    fn prim_equals_kruskal_weight() {
        let g = triangle();
        assert_eq!(
            prims_mst(&g).unwrap().total_weight,
            kruskals_mst(&g).unwrap().total_weight
        );
    }

    #[test]
    fn equal_weight_ties_are_deterministic() {
        // All weight 1; the tie-break selects the lowest (min,max,id) edges.
        let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2, 3], Directedness::Undirected);
        g.add_edge(0, 1, 1).unwrap(); // id 0
        g.add_edge(1, 2, 1).unwrap(); // id 1
        g.add_edge(2, 3, 1).unwrap(); // id 2
        g.add_edge(0, 3, 1).unwrap(); // id 3
        g.add_edge(0, 2, 1).unwrap(); // id 4
        assert_eq!(kruskals_mst(&g).unwrap().edges, vec![0, 3, 4]);
    }

    #[test]
    fn disconnected_graph_fails() {
        let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2, 3], Directedness::Undirected);
        g.add_edge(0, 1, 1).unwrap();
        assert_eq!(kruskals_mst(&g).unwrap_err(), GraphError::Disconnected);
    }

    #[test]
    fn directed_graph_is_wrong_kind() {
        let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1], Directedness::Directed);
        g.add_edge(0, 1, 1).unwrap();
        assert!(matches!(
            kruskals_mst(&g),
            Err(GraphError::WrongGraphKind(_))
        ));
    }

    #[test]
    fn valid_certificate_verifies_tampered_rejected() {
        let g = triangle();
        let cert = kruskals_mst(&g).unwrap().certificate();
        assert!(verify_mst(&g, &cert).is_ok());

        // Tamper with the weight.
        let mut bad = cert.clone();
        bad.total_weight_repr = "99".to_string();
        assert!(verify_mst(&g, &bad).is_err());

        // A suboptimal spanning tree {edge1, edge2} (weight 5) is rejected by
        // the cycle property: edge0 (weight 1) is cheaper than the path max 3.
        let suboptimal = crate::certificate::MstCertificate {
            edge_ids: vec![1, 2],
            total_weight_repr: "5".to_string(),
        };
        assert!(verify_mst(&g, &suboptimal).is_err());
    }
}
