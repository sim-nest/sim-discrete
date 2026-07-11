#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Discrete graph algorithms.
//!
//! This crate hosts graph value types, traversal, connectivity, MST, shortest paths,
//! the graph <-> matrix bridge, and certificate-producing verifiers. All-pairs
//! shortest paths and reachability are thin wrappers over the algebra spine's
//! semiring closure, never re-implemented here.
//!
//! Boundary: depends on `sim-lib-discrete-algebra`; never on `sim-lib-rank`.

pub mod bridge;
pub mod cards;
pub mod certificate;
pub mod connectivity;
pub mod edge;
pub mod error;
pub mod graph;
pub mod intring;
pub mod mst;
pub mod path;
pub mod traversal;
mod unionfind;

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
pub use edge::{Directedness, Edge};
pub use error::GraphError;
pub use graph::{Graph, Neighbor};
pub use intring::IntRing;
pub use mst::{kruskals_mst, prims_mst};
pub use path::{PathResult, all_pairs_shortest_paths, bellman_ford, dijkstra, reachability};
pub use traversal::{Traversal, bfs, dfs};

/// Cookbook recipes for this lib, embedded at build time.
pub static RECIPES: sim_cookbook::EmbeddedDir =
    include!(concat!(env!("OUT_DIR"), "/cookbook_recipes.rs"));
