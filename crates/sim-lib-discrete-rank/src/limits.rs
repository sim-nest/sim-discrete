//! Reviewable limits for rank-space descriptors built from untrusted input.

use crate::error::RankAdapterError;

/// Shared bounds for discrete rank-space construction.
///
/// These limits keep descriptor construction and rank APIs in the same finite
/// domains as the underlying combinatorial iterators. They also cap eager
/// allocations such as simple-graph edge tables before allocation starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscreteRankLimits {
    /// Maximum bit-vector width.
    pub max_bit_vector_width: usize,
    /// Maximum subset ground-set size.
    pub max_subset_size: usize,
    /// Maximum combination ground-set size.
    pub max_combination_n: usize,
    /// Maximum permutation size.
    pub max_permutation_n: usize,
    /// Maximum number of nodes in an eagerly allocated simple-graph space.
    pub max_simple_graph_nodes: usize,
    /// Maximum number of positions in a bounded integer vector.
    pub max_bounded_vector_len: usize,
    /// Maximum radix accepted in bounded-vector and signal descriptors.
    pub max_radix: u64,
    /// Maximum number of coefficients in an FWHT signal descriptor.
    pub max_fwht_signal_len: usize,
    /// Maximum alphabet size for an FWHT signal descriptor.
    pub max_fwht_alphabet: u64,
}

impl DiscreteRankLimits {
    /// The default limit policy used by public read-construct descriptors.
    pub const DEFAULT: Self = Self {
        max_bit_vector_width: 127,
        max_subset_size: 127,
        max_combination_n: 127,
        max_permutation_n: 127,
        max_simple_graph_nodes: 16,
        max_bounded_vector_len: 127,
        max_radix: 1_000_000,
        max_fwht_signal_len: 127,
        max_fwht_alphabet: 1_000_000,
    };

    /// Validate a bit-vector width.
    pub fn check_bit_vector_width(&self, width: usize) -> Result<(), RankAdapterError> {
        self.check_usize("bit-vector width", width, self.max_bit_vector_width)
    }

    /// Validate a subset ground-set size.
    pub fn check_subset_size(&self, n: usize) -> Result<(), RankAdapterError> {
        self.check_usize("subset size", n, self.max_subset_size)
    }

    /// Validate combination `n` and `k`.
    pub fn check_combination(&self, n: usize, k: usize) -> Result<(), RankAdapterError> {
        self.check_usize("combination n", n, self.max_combination_n)?;
        if k > n {
            return Err(RankAdapterError::Invalid(format!(
                "combination k={k} exceeds n={n}"
            )));
        }
        Ok(())
    }

    /// Validate a permutation size.
    pub fn check_permutation_size(&self, n: usize) -> Result<(), RankAdapterError> {
        self.check_usize("permutation size", n, self.max_permutation_n)
    }

    /// Validate a simple-graph node count.
    pub fn check_simple_graph_nodes(&self, n: usize) -> Result<(), RankAdapterError> {
        self.check_usize("simple-graph nodes", n, self.max_simple_graph_nodes)
    }

    /// Validate a bounded integer vector's radix list.
    pub fn check_radices(&self, radices: &[u64]) -> Result<(), RankAdapterError> {
        self.check_usize(
            "bounded-int-vector length",
            radices.len(),
            self.max_bounded_vector_len,
        )?;
        for (index, radix) in radices.iter().copied().enumerate() {
            self.check_radix(index, radix)?;
        }
        Ok(())
    }

    /// Validate FWHT signal rank-space dimensions.
    pub fn check_fwht_signal(&self, len: usize, alphabet: u64) -> Result<(), RankAdapterError> {
        self.check_usize("FWHT signal length", len, self.max_fwht_signal_len)?;
        self.check_bounded_u64("FWHT alphabet", alphabet, self.max_fwht_alphabet)
    }

    fn check_radix(&self, index: usize, radix: u64) -> Result<(), RankAdapterError> {
        if radix == 0 {
            return Err(RankAdapterError::Invalid(format!("radix {index} is zero")));
        }
        self.check_bounded_u64("radix", radix, self.max_radix)
    }

    fn check_usize(
        &self,
        name: &str,
        value: usize,
        maximum: usize,
    ) -> Result<(), RankAdapterError> {
        if value > maximum {
            return Err(RankAdapterError::LimitExceeded(format!(
                "{name} {value} exceeds {maximum}"
            )));
        }
        Ok(())
    }

    fn check_bounded_u64(
        &self,
        name: &str,
        value: u64,
        maximum: u64,
    ) -> Result<(), RankAdapterError> {
        if value == 0 {
            return Err(RankAdapterError::Invalid(format!(
                "{name} must be non-zero"
            )));
        }
        if value > maximum {
            return Err(RankAdapterError::LimitExceeded(format!(
                "{name} {value} exceeds {maximum}"
            )));
        }
        Ok(())
    }
}

impl Default for DiscreteRankLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}
