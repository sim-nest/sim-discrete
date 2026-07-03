//! Graph <-> matrix conversions: adjacency (boolean, min-plus, sparse),
//! incidence, and Laplacian, with explicit multiedge policies and a mapping
//! witness.

use crate::edge::Directedness;
use crate::error::GraphError;
use crate::graph::Graph;
use crate::intring::IntRing;
use sim_lib_discrete_algebra::{BoolRing, Matrix, MinPlus, SparseEntry, SparseMatrix};
use std::collections::HashMap;

/// Resolved canonical-pair values plus the per-edge `(row, col)` mapping.
type ResolveResult =
    Result<(HashMap<(usize, usize), i64>, Vec<Option<(usize, usize)>>), GraphError>;

/// How to collapse parallel edges between the same endpoints into one cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiedgePolicy {
    /// Fail if any pair has more than one edge.
    ErrorOnMultiedge,
    /// Keep the first edge (by edge id) for each pair.
    KeepFirst,
    /// Keep the last edge (by edge id) for each pair.
    KeepLast,
    /// Keep the minimum weight for each pair.
    MinWeight,
    /// Sum the weights for each pair.
    SumWeight,
    /// Use the number of parallel edges as the cell value.
    CountEdges,
}

impl MultiedgePolicy {
    fn label(self) -> &'static str {
        match self {
            MultiedgePolicy::ErrorOnMultiedge => "error-on-multiedge",
            MultiedgePolicy::KeepFirst => "keep-first",
            MultiedgePolicy::KeepLast => "keep-last",
            MultiedgePolicy::MinWeight => "min-weight",
            MultiedgePolicy::SumWeight => "sum-weight",
            MultiedgePolicy::CountEdges => "count-edges",
        }
    }
}

/// Mapping metadata recording how graph elements landed in the matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphMatrixMap {
    /// `node_to_row[node]` is the matrix row for `node` (identity here).
    pub node_to_row: Vec<usize>,
    /// `row_to_node[row]` is the node for a matrix row (identity here).
    pub row_to_node: Vec<usize>,
    /// `edge_to_entry[edge_id]` is the `(row, col)` the edge contributed to.
    pub edge_to_entry: Vec<Option<(usize, usize)>>,
    /// The multiedge policy applied, as a label.
    pub policy: String,
}

fn identity_map(n: usize, edge_count: usize, policy: &str) -> GraphMatrixMap {
    GraphMatrixMap {
        node_to_row: (0..n).collect(),
        row_to_node: (0..n).collect(),
        edge_to_entry: vec![None; edge_count],
        policy: policy.to_string(),
    }
}

fn canonical<N, W>(graph: &Graph<N, W>, s: usize, t: usize) -> (usize, usize) {
    if graph.is_directed() {
        (s, t)
    } else {
        (s.min(t), s.max(t))
    }
}

/// Resolve parallel edges per the policy. Returns the resolved value per
/// canonical pair and the per-edge entry mapping.
fn resolve<N>(graph: &Graph<N, i64>, policy: MultiedgePolicy) -> ResolveResult {
    let mut groups: HashMap<(usize, usize), Vec<(usize, i64)>> = HashMap::new();
    let mut edge_to_entry = vec![None; graph.edge_count()];
    for e in &graph.edges {
        if e.is_self_loop() {
            continue;
        }
        let key = canonical(graph, e.source, e.target);
        groups.entry(key).or_default().push((e.id, e.weight));
        edge_to_entry[e.id] = Some((e.source, e.target));
    }
    let mut resolved = HashMap::new();
    for (key, mut group) in groups {
        group.sort_by_key(|(id, _)| *id);
        if matches!(policy, MultiedgePolicy::ErrorOnMultiedge) && group.len() > 1 {
            return Err(GraphError::Unsupported(
                "multiple edges between a pair under ErrorOnMultiedge".to_string(),
            ));
        }
        let value = match policy {
            MultiedgePolicy::ErrorOnMultiedge | MultiedgePolicy::KeepFirst => group[0].1,
            MultiedgePolicy::KeepLast => group[group.len() - 1].1,
            MultiedgePolicy::MinWeight => group.iter().map(|(_, w)| *w).min().unwrap(),
            MultiedgePolicy::SumWeight => group.iter().map(|(_, w)| *w).sum(),
            MultiedgePolicy::CountEdges => group.len() as i64,
        };
        resolved.insert(key, value);
    }
    Ok((resolved, edge_to_entry))
}

/// Expand a canonical resolved map into directed `(row, col)` cells.
fn directed_cells<N>(
    graph: &Graph<N, i64>,
    resolved: &HashMap<(usize, usize), i64>,
) -> Vec<((usize, usize), i64)> {
    let mut cells = Vec::new();
    for (&(a, b), &v) in resolved {
        cells.push(((a, b), v));
        if !graph.is_directed() {
            cells.push(((b, a), v));
        }
    }
    cells
}

/// Boolean adjacency: `true` where at least one edge connects the pair.
pub fn graph_to_bool_adjacency<N, W>(
    graph: &Graph<N, W>,
) -> Result<(Matrix<BoolRing>, GraphMatrixMap), GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    let mut m = Matrix::filled(n, n, BoolRing(false));
    let undirected = !graph.is_directed();
    let mut map = identity_map(n, graph.edge_count(), "boolean");
    for e in &graph.edges {
        if e.is_self_loop() {
            continue;
        }
        m.data[e.source * n + e.target] = BoolRing(true);
        map.edge_to_entry[e.id] = Some((e.source, e.target));
        if undirected {
            m.data[e.target * n + e.source] = BoolRing(true);
        }
    }
    Ok((m, map))
}

/// Min-plus weighted adjacency (`Inf` = no edge), applying a multiedge policy.
pub fn graph_to_minplus_adjacency<N>(
    graph: &Graph<N, i64>,
    policy: MultiedgePolicy,
) -> Result<(Matrix<MinPlus>, GraphMatrixMap), GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    let (resolved, edge_to_entry) = resolve(graph, policy)?;
    let mut m = Matrix::filled(n, n, MinPlus::Inf);
    for ((r, c), v) in directed_cells(graph, &resolved) {
        m.data[r * n + c] = MinPlus::Fin(v);
    }
    let mut map = identity_map(n, graph.edge_count(), policy.label());
    map.edge_to_entry = edge_to_entry;
    Ok((m, map))
}

/// Sparse min-plus weighted adjacency, applying a multiedge policy.
pub fn graph_to_sparse_adjacency<N>(
    graph: &Graph<N, i64>,
    policy: MultiedgePolicy,
) -> Result<(SparseMatrix<MinPlus>, GraphMatrixMap), GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    let (resolved, edge_to_entry) = resolve(graph, policy)?;
    let mut entries = Vec::new();
    for ((r, c), v) in directed_cells(graph, &resolved) {
        entries.push(SparseEntry {
            row: r,
            col: c,
            value: MinPlus::Fin(v),
        });
    }
    let sparse = SparseMatrix::from_entries(n, n, entries)
        .map_err(|e| GraphError::Unsupported(e.to_string()))?;
    let mut map = identity_map(n, graph.edge_count(), policy.label());
    map.edge_to_entry = edge_to_entry;
    Ok((sparse, map))
}

/// Incidence matrix as a sparse `IntRing` matrix. Directed: `-1` at the source,
/// `+1` at the target. Undirected: `+1` at both endpoints. One column per edge
/// (self-loops omitted).
pub fn graph_to_incidence<N, W>(graph: &Graph<N, W>) -> Result<SparseMatrix<IntRing>, GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    let directed = graph.is_directed();
    let mut entries = Vec::new();
    let mut col = 0;
    for e in &graph.edges {
        if e.is_self_loop() {
            continue;
        }
        if directed {
            entries.push(SparseEntry {
                row: e.source,
                col,
                value: IntRing(-1),
            });
            entries.push(SparseEntry {
                row: e.target,
                col,
                value: IntRing(1),
            });
        } else {
            entries.push(SparseEntry {
                row: e.source,
                col,
                value: IntRing(1),
            });
            entries.push(SparseEntry {
                row: e.target,
                col,
                value: IntRing(1),
            });
        }
        col += 1;
    }
    SparseMatrix::from_entries(n, col, entries).map_err(|e| GraphError::Unsupported(e.to_string()))
}

/// Unweighted graph Laplacian `L = D - A` over `IntRing`, ignoring self-loops.
/// For an undirected graph each row sums to zero.
pub fn graph_to_laplacian<N, W>(graph: &Graph<N, W>) -> Result<Matrix<IntRing>, GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    let mut m = Matrix::filled(n, n, IntRing(0));
    let undirected = !graph.is_directed();
    for e in &graph.edges {
        if e.is_self_loop() {
            continue;
        }
        m.data[e.source * n + e.target].0 -= 1;
        m.data[e.source * n + e.source].0 += 1;
        if undirected {
            m.data[e.target * n + e.source].0 -= 1;
            m.data[e.target * n + e.target].0 += 1;
        }
    }
    Ok(m)
}

/// Reconstruct a graph from a min-plus adjacency matrix (`Inf` = no edge). Node
/// labels are their indices. Directedness is supplied by the caller.
pub fn minplus_adjacency_to_graph(
    matrix: &Matrix<MinPlus>,
    directedness: Directedness,
) -> Result<Graph<usize, i64>, GraphError> {
    if !matrix.is_square() {
        return Err(GraphError::Unsupported(
            "adjacency must be square".to_string(),
        ));
    }
    let n = matrix.rows;
    let mut g = Graph::with_nodes((0..n).collect(), directedness);
    let undirected = matches!(directedness, Directedness::Undirected);
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            if undirected && j < i {
                continue;
            }
            if let MinPlus::Fin(w) = matrix.data[i * n + j] {
                g.add_edge(i, j, w)?;
            }
        }
    }
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weighted_directed() -> Graph<usize, i64> {
        let mut g = Graph::with_nodes(vec![0, 1, 2], Directedness::Directed);
        g.add_edge(0, 1, 4).unwrap();
        g.add_edge(1, 2, 7).unwrap();
        g
    }

    #[test]
    fn adjacency_round_trip_preserves_simple_graph() {
        let g = weighted_directed();
        let (m, _map) = graph_to_minplus_adjacency(&g, MultiedgePolicy::ErrorOnMultiedge).unwrap();
        let back = minplus_adjacency_to_graph(&m, Directedness::Directed).unwrap();
        let mut got: Vec<_> = back
            .edges
            .iter()
            .map(|e| (e.source, e.target, e.weight))
            .collect();
        got.sort_unstable();
        assert_eq!(got, vec![(0, 1, 4), (1, 2, 7)]);
        assert!(back.is_directed());
    }

    #[test]
    fn directedness_is_preserved() {
        let mut g: Graph<usize, i64> = Graph::with_nodes(vec![0, 1], Directedness::Undirected);
        g.add_edge(0, 1, 5).unwrap();
        let (m, _) = graph_to_minplus_adjacency(&g, MultiedgePolicy::MinWeight).unwrap();
        // Symmetric for undirected: cells (0,1) and (1,0) in a 2x2 matrix.
        assert_eq!(m.data[1], MinPlus::Fin(5));
        assert_eq!(m.data[2], MinPlus::Fin(5));
    }

    #[test]
    fn multiedge_policies_differ() {
        let mut g: Graph<usize, i64> = Graph::with_nodes(vec![0, 1], Directedness::Directed);
        g.add_edge(0, 1, 3).unwrap();
        g.add_edge(0, 1, 10).unwrap();
        let cell = |p| graph_to_minplus_adjacency(&g, p).unwrap().0.data[1];
        assert_eq!(cell(MultiedgePolicy::MinWeight), MinPlus::Fin(3));
        assert_eq!(cell(MultiedgePolicy::SumWeight), MinPlus::Fin(13));
        assert_eq!(cell(MultiedgePolicy::KeepLast), MinPlus::Fin(10));
        assert_eq!(cell(MultiedgePolicy::CountEdges), MinPlus::Fin(2));
        assert!(graph_to_minplus_adjacency(&g, MultiedgePolicy::ErrorOnMultiedge).is_err());
    }

    #[test]
    fn laplacian_rows_sum_to_zero() {
        // Triangle: each row of L sums to zero for a connected undirected graph.
        let mut g: Graph<usize, i64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Undirected);
        g.add_edge(0, 1, 1).unwrap();
        g.add_edge(1, 2, 1).unwrap();
        g.add_edge(0, 2, 1).unwrap();
        let l = graph_to_laplacian(&g).unwrap();
        for r in 0..3 {
            let sum: i64 = (0..3).map(|c| l.data[r * 3 + c].0).sum();
            assert_eq!(sum, 0, "row {r}");
        }
        // Diagonal is the degree (2 for the triangle).
        assert_eq!(l.data[0].0, 2);
    }

    #[test]
    fn incidence_columns_have_expected_signs() {
        let g = weighted_directed();
        let inc = graph_to_incidence(&g).unwrap();
        // Column 0 is edge 0->1: -1 at row 0, +1 at row 1.
        let col0: Vec<_> = inc.entries.iter().filter(|e| e.col == 0).collect();
        assert!(col0.iter().any(|e| e.row == 0 && e.value == IntRing(-1)));
        assert!(col0.iter().any(|e| e.row == 1 && e.value == IntRing(1)));
    }
}
