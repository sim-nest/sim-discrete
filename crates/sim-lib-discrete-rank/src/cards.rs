//! Card content for the discrete rank spaces. The facade install turns these
//! into kernel claims via `publish_space_claims` (which needs a `Cx`).

use crate::descriptor::CardSpec;

/// The discrete rank-space cards available so far.
pub fn discrete_rank_cards() -> &'static [CardSpec] {
    const CARDS: &[CardSpec] = &[
        CardSpec {
            key: "rank/discrete/bit-vector",
            summary: "Fixed-width bit vectors.",
            order: "natural-binary",
            metric: "hamming",
        },
        CardSpec {
            key: "rank/discrete/subset",
            summary: "Subsets of a finite ground set.",
            order: "bitmask",
            metric: "symmetric-difference",
        },
        CardSpec {
            key: "rank/discrete/combination",
            summary: "k-subsets of n.",
            order: "combinadic",
            metric: "symmetric-difference",
        },
        CardSpec {
            key: "rank/discrete/permutation",
            summary: "Permutations of n elements.",
            order: "lehmer",
            metric: "kendall-tau",
        },
        CardSpec {
            key: "rank/discrete/bounded-int-vector",
            summary: "Mixed-radix integer vectors.",
            order: "mixed-radix",
            metric: "l1",
        },
        CardSpec {
            key: "rank/discrete/simple-graph",
            summary: "Labeled simple undirected graphs on n nodes.",
            order: "adjacency-upper-triangle",
            metric: "edge-symmetric-difference",
        },
        CardSpec {
            key: "rank/discrete/fwht-signal",
            summary: "Finite-alphabet integer signals.",
            order: "mixed-radix",
            metric: "coefficient-l1",
        },
    ];
    CARDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_spaces_published() {
        assert_eq!(discrete_rank_cards().len(), 7);
        for card in discrete_rank_cards() {
            assert!(card.key.starts_with("rank/discrete/"));
        }
    }
}
