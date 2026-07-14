//! Deterministic cookbook builders for discrete combinatorics recipes.

use crate::{
    CombError, combination_rank, combination_unrank, permutation_rank, permutation_unrank,
};

/// Report produced by the rankable values cookbook recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankableValuesDemo {
    /// Combination value used by the recipe.
    pub combination: Vec<usize>,
    /// Combination rank rendered as a decimal ordinal.
    pub combination_rank: String,
    /// Combination recovered from the ordinal.
    pub combination_unranked: Vec<usize>,
    /// Permutation value used by the recipe.
    pub permutation: Vec<usize>,
    /// Permutation rank rendered as a decimal ordinal.
    pub permutation_rank: String,
    /// Permutation recovered from the ordinal.
    pub permutation_unranked: Vec<usize>,
}

/// Build the modeled combination/permutation rank report used by the cookbook.
pub fn rankable_values_demo() -> Result<RankableValuesDemo, CombError> {
    let combination = vec![0, 2, 4];
    let combination_rank = combination_rank(&combination, 5)?;
    let combination_unranked = combination_unrank(&combination_rank, 5, 3)?;

    let permutation = vec![2, 0, 1];
    let permutation_rank = permutation_rank(&permutation)?;
    let permutation_unranked = permutation_unrank(&permutation_rank, 3)?;

    Ok(RankableValuesDemo {
        combination,
        combination_rank: combination_rank.to_string(),
        combination_unranked,
        permutation,
        permutation_rank: permutation_rank.to_string(),
        permutation_unranked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rankable_values_round_trip_ordinals() {
        let demo = rankable_values_demo().expect("valid rankable values demo");

        assert_eq!(demo.combination_unranked, demo.combination);
        assert_eq!(demo.permutation_unranked, demo.permutation);
        assert_eq!(demo.combination_rank, "4");
        assert_eq!(demo.permutation_rank, "4");
    }
}
