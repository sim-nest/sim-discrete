#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Bounded deterministic discrete search.
//!
//! This crate supplies a small state-space search contract for discrete
//! algorithms that need deterministic ordering, pruning, propagation, explicit
//! work charging, and receipts. It is generic over the problem state and output:
//! graph, ranking, music, and planning code provide the domain behavior while
//! this crate owns the bounded exploration loop.

pub mod control;
pub mod cookbook;
pub mod engine;
pub mod error;
pub mod problem;
pub mod receipt;
pub mod two_stack;
pub mod word;

pub use control::{SearchControl, SearchOrder, WorkCosts};
pub use cookbook::{ConstrainedWordDemo, constrained_word_demo};
pub use engine::solve;
pub use error::SearchError;
pub use problem::{NeverInterrupt, SearchInterrupt, SearchProblem, SearchStep};
pub use receipt::{SearchReceipt, SearchRun, SearchStatus};
pub use two_stack::TwoStackAdapter;
pub use word::{
    ConstrainedWordProblem, WordSearchSolution, WordSearchState, render_constrained_word_demo,
};

/// Cookbook recipes for this lib, embedded at build time.
pub static RECIPES: sim_cookbook::EmbeddedDir =
    include!(concat!(env!("OUT_DIR"), "/cookbook_recipes.rs"));
