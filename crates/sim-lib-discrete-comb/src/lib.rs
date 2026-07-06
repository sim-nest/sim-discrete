#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Discrete combinatorics.
//!
//! This crate hosts exact counting functions (over `num_bigint::BigUint`), lazy
//! enumerators, and canonical combinadic / Lehmer / mixed-radix rank-unrank
//! helpers. These ordinals are the bridge to rank, but this crate never depends
//! on `sim-lib-rank`.

pub mod bit_vector;
pub mod cards;
pub mod combination;
pub mod count;
pub mod error;
pub mod mixed_radix;
pub mod partition;
pub mod permutation;
pub mod subset;

pub use bit_vector::{BitVectorIter, bit_vector_rank, bit_vector_unrank, bit_vectors};
pub use cards::{CardSpec, combinatorics_cards};
pub use combination::{CombinationIter, combination_rank, combination_unrank, combinations};
pub use count::{
    MAX_BINOMIAL_INPUT, MAX_FACTORIAL_INPUT, MAX_PARTITION_INPUT, bell_number, binomial,
    binomial_checked, factorial, factorial_checked, falling_factorial, integer_partition_count,
    integer_partition_count_checked, multinomial, permutation_count, stirling2,
};
pub use error::CombError;
pub use mixed_radix::{mixed_radix_rank, mixed_radix_unrank};
pub use partition::{IntegerPartitionIter, integer_partitions};
pub use permutation::{PermutationIter, permutation_rank, permutation_unrank, permutations};
pub use subset::{SubsetIter, subset_rank, subset_unrank, subsets};
