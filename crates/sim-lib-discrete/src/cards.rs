//! The unified browse index for the discrete-math family.
//!
//! This is the front-page card list a runtime install publishes. Each family
//! crate owns its own detailed `CardSpec`s; this index is the kernel-free
//! catalogue the facade exposes so browse can enumerate the whole family at once.

/// A one-line browse entry for a discrete family or rank space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscreteCard {
    /// Stable card key.
    pub key: &'static str,
    /// One-line summary.
    pub summary: &'static str,
}

/// Every discrete family and rank space, as a single browseable catalogue.
pub fn discrete_cards() -> &'static [DiscreteCard] {
    const CARDS: &[DiscreteCard] = &[
        DiscreteCard {
            key: "discrete/algebra",
            summary: "Semiring matrices, powers, and Kleene closure (the spine).",
        },
        DiscreteCard {
            key: "discrete/graph",
            summary: "Graphs, MST, shortest paths, and certificates.",
        },
        DiscreteCard {
            key: "discrete/combinatorics",
            summary: "Exact counts, lazy enumerators, and canonical ordinals.",
        },
        DiscreteCard {
            key: "discrete/spectral",
            summary: "FWHT, XOR/subset convolution, and Walsh signatures.",
        },
        DiscreteCard {
            key: "discrete/rank",
            summary: "Finite rank spaces, metrics, and invariant grade compilers.",
        },
        DiscreteCard {
            key: "discrete/music-adapter",
            summary: "FWHT-based melody analysis (consumes the spectral atlas).",
        },
    ];
    CARDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_is_complete_and_ascii() {
        let cards = discrete_cards();
        assert_eq!(cards.len(), 6);
        for card in cards {
            assert!(card.key.starts_with("discrete/"));
            assert!(card.key.is_ascii() && card.summary.is_ascii());
        }
    }
}
