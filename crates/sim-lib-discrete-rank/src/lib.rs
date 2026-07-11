#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Discrete rank adapters.
//!
//! This is the only discrete crate allowed to depend on `sim-lib-rank`. It hosts finite
//! rank spaces (bit-vector, subset, combination, permutation, simple-graph,
//! FWHT-signal), rank metrics, and grade compilers that synthesize a rank grade
//! from algebraic / spectral invariants (MST weight, closure density, Walsh
//! spectral entropy).
//!
//! Boundary: `sim-lib-rank` must never depend on this crate; the dependency is
//! one-way.

pub mod cards;
pub mod combinatorial;
pub mod descriptor;
pub mod error;
pub mod grade;
pub mod graph_space;
pub mod lattice;
pub mod metric;
pub mod signal_space;
pub mod tree_metric;

pub use cards::discrete_rank_cards;
pub use combinatorial::{BoundedIntVectorSpace, CombinationSpace, PermutationSpace};
pub use descriptor::{CardSpec, SpaceDescriptor};
pub use error::RankAdapterError;
pub use grade::{
    closure_density_grade, mst_weight_grade, signal_spectral_entropy_grade, spectral_entropy_band,
};
pub use graph_space::SimpleGraphSpace;
pub use lattice::{BitVectorSpace, SubsetSpace};
pub use signal_space::FwhtSignalSpace;
pub use tree_metric::spanning_tree_swap_distance;

/// Cookbook recipes for this lib, embedded at build time.
pub static RECIPES: sim_cookbook::EmbeddedDir =
    include!(concat!(env!("OUT_DIR"), "/cookbook_recipes.rs"));
