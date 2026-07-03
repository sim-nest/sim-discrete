//! Browse/help card content for the graph family, as kernel-free static data.
//!
//! These descriptors are the single source of truth for the graph Cards. The
//! facade crate registers them into the runtime; keeping them here as plain data
//! preserves the boundary (this crate never depends on the kernel).

/// A browse/help card descriptor for one discrete family or operation group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardSpec {
    /// Stable card key (for example `discrete/graph`).
    pub key: &'static str,
    /// One-line summary.
    pub summary: &'static str,
    /// Operation names exposed by this family.
    pub operations: &'static [&'static str],
    /// Data forms / value types this family deals in.
    pub data_forms: &'static [&'static str],
    /// Known limits and capability needs, in prose.
    pub limits: &'static str,
}

/// The graph family cards available so far.
pub fn graph_cards() -> &'static [CardSpec] {
    const CARDS: &[CardSpec] = &[CardSpec {
        key: "discrete/graph",
        summary: "Weighted graphs with traversal and connectivity.",
        operations: &[
            "bfs",
            "dfs",
            "connected-components",
            "weakly-connected-components",
            "strongly-connected-components",
        ],
        data_forms: &["graph", "edge"],
        limits: "Node identity is index-based; multiedges and self-loops are \
                 representable. Connectivity validates endpoints and fails closed \
                 on out-of-range nodes.",
    }];
    CARDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_card_is_present_and_ascii() {
        let cards = graph_cards();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].key, "discrete/graph");
        assert!(cards[0].summary.is_ascii());
    }
}
