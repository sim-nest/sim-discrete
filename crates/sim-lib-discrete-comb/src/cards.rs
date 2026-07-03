//! Browse/help card content for the combinatorics family, as static data.

/// A browse/help card descriptor for the combinatorics family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardSpec {
    /// Stable card key.
    pub key: &'static str,
    /// One-line summary.
    pub summary: &'static str,
    /// Operation names exposed by this family.
    pub operations: &'static [&'static str],
    /// Data forms / value types this family deals in.
    pub data_forms: &'static [&'static str],
    /// Known limits and capability needs.
    pub limits: &'static str,
}

/// The combinatorics family cards.
pub fn combinatorics_cards() -> &'static [CardSpec] {
    const CARDS: &[CardSpec] = &[CardSpec {
        key: "discrete/combinatorics",
        summary: "Exact counts, lazy enumerators, and canonical ordinals.",
        operations: &[
            "factorial",
            "binomial",
            "stirling2",
            "bell-number",
            "integer-partition-count",
            "subsets",
            "combinations",
            "permutations",
            "integer-partitions",
            "rank-unrank",
        ],
        data_forms: &[
            "bit-vector",
            "subset",
            "combination",
            "permutation",
            "partition",
        ],
        limits: "Counts are exact BigUint. Bitmask iterators cap cardinality at \
                 127; explosive enumerations require explicit bounds from the \
                 caller via take/limit.",
    }];
    CARDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_present() {
        assert_eq!(combinatorics_cards()[0].key, "discrete/combinatorics");
    }
}
