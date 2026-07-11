#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Discrete algebra spine.
//!
//! This crate hosts the [`Semiring`] trait, the standard
//! semiring instances, and a generic dense / coordinate-list sparse matrix with
//! power and Kleene-closure engines. Graph reachability and all-pairs shortest
//! paths are *derived* from these (boolean closure, min-plus closure, counting
//! powers) rather than re-implemented per algorithm.
//!
//! Boundary: depends only on `thiserror` (and later `num-bigint`). It must never
//! depend on `sim-lib-rank` or any music crate.

pub mod boolean;
pub mod closure;
pub mod counting;
pub mod error;
pub mod gf2;
pub mod matrix;
pub mod power;
pub mod real;
pub mod semiring;
pub mod sparse;
pub mod tropical_max;
pub mod tropical_min;
pub mod views;

pub use boolean::BoolRing;
pub use counting::Counting;
pub use error::AlgebraError;
pub use gf2::Gf2;
pub use matrix::{AlgebraLimits, Matrix};
pub use real::RealF64;
pub use semiring::Semiring;
pub use sparse::{SparseEntry, SparseMatrix};
pub use tropical_max::MaxPlus;
pub use tropical_min::MinPlus;
pub use views::{DiagonalView, IdentityView, PermutationView};

/// Cookbook recipes for this lib, embedded at build time.
pub static RECIPES: sim_cookbook::EmbeddedDir =
    include!(concat!(env!("OUT_DIR"), "/cookbook_recipes.rs"));
