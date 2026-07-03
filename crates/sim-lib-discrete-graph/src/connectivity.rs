//! Connected components (undirected / weak) and strongly connected components.

use crate::edge::Directedness;
use crate::error::GraphError;
use crate::graph::Graph;

/// Build adjacency lists, optionally treating every edge as undirected.
fn adjacency<N, W>(
    graph: &Graph<N, W>,
    force_undirected: bool,
) -> Result<Vec<Vec<usize>>, GraphError> {
    graph.validate()?;
    let n = graph.node_count();
    let mut adj = vec![Vec::new(); n];
    let undirected = force_undirected || matches!(graph.directedness, Directedness::Undirected);
    for e in &graph.edges {
        adj[e.source].push(e.target);
        if undirected {
            adj[e.target].push(e.source);
        }
    }
    for list in &mut adj {
        list.sort_unstable();
    }
    Ok(adj)
}

/// Flood fill over undirected adjacency, returning sorted components ordered by
/// their smallest member.
fn flood(adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adj.len();
    let mut seen = vec![false; n];
    let mut comps = Vec::new();
    for s in 0..n {
        if seen[s] {
            continue;
        }
        let mut members = Vec::new();
        let mut stack = vec![s];
        seen[s] = true;
        while let Some(u) = stack.pop() {
            members.push(u);
            for &w in &adj[u] {
                if !seen[w] {
                    seen[w] = true;
                    stack.push(w);
                }
            }
        }
        members.sort_unstable();
        comps.push(members);
    }
    comps
}

/// Connected components of an undirected graph (treats edges as undirected).
pub fn connected_components<N, W>(graph: &Graph<N, W>) -> Result<Vec<Vec<usize>>, GraphError> {
    Ok(flood(&adjacency(graph, true)?))
}

/// Weakly connected components of a directed graph (ignores edge direction).
pub fn weakly_connected_components<N, W>(
    graph: &Graph<N, W>,
) -> Result<Vec<Vec<usize>>, GraphError> {
    Ok(flood(&adjacency(graph, true)?))
}

/// Strongly connected components via iterative Tarjan, respecting edge
/// direction. Each component is sorted; components are ordered by smallest
/// member. (For undirected graphs this coincides with connected components.)
pub fn strongly_connected_components<N, W>(
    graph: &Graph<N, W>,
) -> Result<Vec<Vec<usize>>, GraphError> {
    let adj = adjacency(graph, false)?;
    let n = adj.len();
    const UNVISITED: usize = usize::MAX;
    let mut index = vec![UNVISITED; n];
    let mut low = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut tstack: Vec<usize> = Vec::new();
    let mut comps: Vec<Vec<usize>> = Vec::new();
    let mut counter = 0usize;

    for start in 0..n {
        if index[start] != UNVISITED {
            continue;
        }
        // Work stack of (node, next-child-cursor).
        let mut work: Vec<(usize, usize)> = vec![(start, 0)];
        while let Some(&(v, ci)) = work.last() {
            if ci == 0 && index[v] == UNVISITED {
                index[v] = counter;
                low[v] = counter;
                counter += 1;
                tstack.push(v);
                on_stack[v] = true;
            }
            if ci < adj[v].len() {
                let w = adj[v][ci];
                work.last_mut().unwrap().1 += 1;
                if index[w] == UNVISITED {
                    work.push((w, 0));
                } else if on_stack[w] {
                    low[v] = low[v].min(index[w]);
                }
            } else {
                if low[v] == index[v] {
                    let mut comp = Vec::new();
                    loop {
                        let w = tstack.pop().expect("tarjan stack non-empty");
                        on_stack[w] = false;
                        comp.push(w);
                        if w == v {
                            break;
                        }
                    }
                    comp.sort_unstable();
                    comps.push(comp);
                }
                work.pop();
                if let Some(&(parent, _)) = work.last() {
                    low[parent] = low[parent].min(low[v]);
                }
            }
        }
    }
    comps.sort_by_key(|c| c[0]);
    Ok(comps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undirected_components_match_fixture() {
        // Two clusters: {0,1,2} and {3,4}; node 5 isolated.
        let mut g: Graph<u8, u64> =
            Graph::with_nodes(vec![0, 1, 2, 3, 4, 5], Directedness::Undirected);
        g.add_edge(0, 1, 1).unwrap();
        g.add_edge(1, 2, 1).unwrap();
        g.add_edge(3, 4, 1).unwrap();
        let comps = connected_components(&g).unwrap();
        assert_eq!(comps, vec![vec![0, 1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn scc_finds_directed_cycle() {
        // 0->1->2->0 is one SCC; 3 is its own; edge 2->3 does not merge them.
        let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2, 3], Directedness::Directed);
        g.add_edge(0, 1, 1).unwrap();
        g.add_edge(1, 2, 1).unwrap();
        g.add_edge(2, 0, 1).unwrap();
        g.add_edge(2, 3, 1).unwrap();
        let comps = strongly_connected_components(&g).unwrap();
        assert_eq!(comps, vec![vec![0, 1, 2], vec![3]]);
    }

    #[test]
    fn scc_of_dag_is_singletons() {
        let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Directed);
        g.add_edge(0, 1, 1).unwrap();
        g.add_edge(1, 2, 1).unwrap();
        let comps = strongly_connected_components(&g).unwrap();
        assert_eq!(comps, vec![vec![0], vec![1], vec![2]]);
    }
}
