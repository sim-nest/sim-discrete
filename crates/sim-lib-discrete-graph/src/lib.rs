#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Discrete graph algorithms.
//!
//! This crate hosts graph value types, traversal, connectivity, MST, shortest
//! paths, minimum-cost bipartite assignment, staged dynamic programming,
//! edit/DTW alignment, the graph <-> matrix bridge, and certificate-producing
//! verifiers. Expensive assignment and dynamic-programming APIs expose explicit
//! cell/edge work, memory, deadline, and cancellation control. All-pairs
//! shortest paths and reachability are thin wrappers over the algebra spine's
//! semiring closure, never re-implemented here.
//!
//! Boundary: depends on `sim-lib-discrete-algebra`; never on `sim-lib-rank`.

pub mod alignment;
pub mod assignment;
pub mod bridge;
pub mod cards;
pub mod certificate;
pub mod connectivity;
pub mod control;
pub mod cookbook;
pub mod cost;
pub mod edge;
pub mod error;
pub mod graph;
pub mod intring;
pub mod layered;
pub mod mst;
pub mod path;
pub mod traversal;
mod unionfind;

pub use alignment::{
    Alignment, AlignmentBoundary, AlignmentCell, AlignmentCertificate, AlignmentMemory,
    AlignmentMove, AlignmentStep, AlignmentWindow, DtwPolicy, GapPolicy, dynamic_time_warp,
    dynamic_time_warp_with_control, verify_alignment,
};
pub use assignment::{
    Assignment, AssignmentCertificate, AssignmentCost, AssignmentOperation, AssignmentPolicy,
    CostMatrix, DoublingPolicy, VoiceCrossingPolicy, min_cost_assignment,
    min_cost_assignment_with_control, verify_assignment,
};
pub use bridge::{
    GraphMatrixMap, MultiedgePolicy, graph_to_bool_adjacency, graph_to_incidence,
    graph_to_laplacian, graph_to_minplus_adjacency, graph_to_sparse_adjacency,
    minplus_adjacency_to_graph,
};
pub use cards::CardSpec;
pub use certificate::{
    MstCertificate, ShortestPathCertificate, SpanningTree, verify_mst, verify_shortest_paths,
};
pub use connectivity::{
    connected_components, strongly_connected_components, weakly_connected_components,
};
pub use control::{
    AlgorithmControl, AlgorithmInterrupt, AlgorithmReceipt, AlgorithmWorkCosts, NeverInterrupt,
};
pub use cookbook::{TinyGraphDemo, tiny_graph_demo};
pub use cost::FiniteCost;
pub use edge::{Directedness, Edge};
pub use error::GraphError;
pub use graph::{Graph, Neighbor};
pub use intring::IntRing;
pub use layered::{
    LayeredCell, LayeredCertificate, LayeredPath, layered_shortest_path,
    layered_shortest_path_with_control, verify_layered_path,
};
pub use mst::{MstWeight, kruskals_mst, prims_mst};
pub use path::{
    PathResult, ShortestPath, all_pairs_shortest_paths, bellman_ford, dijkstra, reachability,
    shortest_path,
};
pub use traversal::{Traversal, bfs, dfs};

/// Cookbook recipes for this lib, embedded at build time.
pub static RECIPES: sim_cookbook::EmbeddedDir =
    include!(concat!(env!("OUT_DIR"), "/cookbook_recipes.rs"));
