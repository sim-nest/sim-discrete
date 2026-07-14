//! Deterministic cookbook builders for discrete rank recipes.

use crate::descriptor::from_nat;
use crate::{CombinationSpace, RankAdapterError};

/// Report produced by the combination-space cookbook recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombinationSpaceDemo {
    /// Rank-space descriptor id.
    pub descriptor_id: &'static str,
    /// Ground-set size.
    pub n: usize,
    /// Combination size.
    pub k: usize,
    /// Combination selected by the recipe.
    pub combination: Vec<usize>,
    /// Rank rendered as a decimal ordinal.
    pub rank: String,
    /// Combination recovered from the ordinal.
    pub unranked: Vec<usize>,
}

/// Build the modeled combination-space rank report used by the cookbook.
pub fn combination_space_demo() -> Result<CombinationSpaceDemo, RankAdapterError> {
    let space = CombinationSpace { n: 6, k: 3 };
    let descriptor = space.descriptor();
    let combination = vec![0, 2, 4];
    let rank = space.rank(&combination)?;
    let unranked = space.unrank(&rank)?;

    Ok(CombinationSpaceDemo {
        descriptor_id: descriptor.id,
        n: space.n,
        k: space.k,
        combination,
        rank: from_nat(&rank).to_string(),
        unranked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_lib_rank::Nat;

    #[test]
    fn combination_space_demo_round_trips_rank() {
        let demo = combination_space_demo().expect("valid combination space demo");

        assert_eq!(demo.descriptor_id, "rank/discrete/combination");
        assert_eq!(demo.unranked, demo.combination);
        let ordinal = demo.rank.parse::<u64>().expect("rank is a u64 fixture");
        assert_eq!(
            CombinationSpace {
                n: demo.n,
                k: demo.k
            }
            .unrank(&Nat::from(ordinal))
            .unwrap(),
            demo.combination
        );
    }
}
