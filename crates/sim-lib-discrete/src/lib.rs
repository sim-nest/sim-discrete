#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Discrete-math family facade.
//!
//! Thin facade crate that re-exports the discrete sub-crates behind features and
//! hosts runtime / Lisp install helpers. Each sub-module is gated by its feature
//! so callers pull in only what they enable.
//!
//! - `algebra`  -> the semiring spine and matrix-closure engine
//! - `graph`    -> graph algorithms, MST, shortest paths, certificates
//! - `comb`     -> combinatorics counts, enumerators, ordinals
//! - `spectral` -> FWHT and the Walsh-domain atlas
//! - `rank`     -> optional rank adapters (requires `sim-lib-rank`)
//!
//! The kernel-free [`forms`] (read-construct codecs) and [`cards`] (browse
//! index) modules are always available; the live kernel `Lib`/`Cx` op and claim
//! registration is provided by the `runtime` feature.

pub mod cards;
#[cfg(feature = "citizen")]
mod citizen;
#[cfg(feature = "runtime")]
mod claims;
pub mod cookbook;
pub mod forms;

/// Cookbook recipes for the discrete domain, embedded at build time.
pub static RECIPES: sim_cookbook::EmbeddedDir =
    include!(concat!(env!("OUT_DIR"), "/cookbook_recipes.rs"));

#[cfg(feature = "runtime")]
pub mod runtime;

pub use cards::{DiscreteCard, discrete_cards};
#[cfg(feature = "citizen")]
pub use citizen::*;
pub use cookbook::matrix_runtime_demo;

#[cfg(feature = "runtime")]
pub use runtime::{DiscreteLib, DiscreteOp, OpKind, install_discrete_lib};

#[cfg(feature = "algebra")]
pub use sim_lib_discrete_algebra as algebra;

#[cfg(feature = "graph")]
pub use sim_lib_discrete_graph as graph;

#[cfg(feature = "comb")]
pub use sim_lib_discrete_comb as comb;

#[cfg(feature = "spectral")]
pub use sim_lib_discrete_spectral as spectral;

#[cfg(feature = "rank")]
pub use sim_lib_discrete_rank as rank;

#[cfg(all(test, feature = "citizen"))]
mod citizen_tests;
