//! Deterministic cookbook builders for discrete combinatorics recipes.

use crate::{
    CombError, canonical_cycles, combination_rank, combination_unrank, longest_only,
    permutation_rank, permutation_unrank, word_rank, word_to_digits, words,
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

/// Report produced by the finite enumeration cookbook recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FiniteEnumerationDemo {
    /// Alphabet supplied by the caller.
    pub alphabet: Vec<String>,
    /// Alphabet after applying the supplied order.
    pub ordered_alphabet: Vec<String>,
    /// Fixed word length supplied by the caller.
    pub length: usize,
    /// Number of words requested by the caller.
    pub limit: usize,
    /// Exact finite catalog size rendered as a decimal integer.
    pub total_words: String,
    /// First words emitted by the lazy iterator.
    pub first_words: Vec<Vec<String>>,
    /// Mixed-radix digits for each emitted word.
    pub first_digits: Vec<Vec<u64>>,
    /// Mixed-radix ranks for each emitted word.
    pub first_ranks: Vec<String>,
    /// Unique canonical rotations for one non-constant emitted word.
    pub canonical_cycles: Vec<Vec<String>>,
    /// Longest emitted words under the word-length selector.
    pub longest_words: Vec<Vec<String>>,
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

/// Build the fixed-alphabet finite enumeration report used by the cookbook.
pub fn finite_enumeration_demo(
    alphabet: Vec<String>,
    length: usize,
    order: Vec<usize>,
    limit: usize,
) -> Result<FiniteEnumerationDemo, CombError> {
    let ordered_alphabet = ordered_alphabet(&alphabet, &order)?;
    let iterator = words(&ordered_alphabet, length);
    let total_words = iterator.total_ordinals().to_string();
    let first_words = iterator.take(limit).collect::<Vec<_>>();
    let first_digits = first_words
        .iter()
        .map(|word| word_to_digits(&ordered_alphabet, word))
        .collect::<Result<Vec<_>, _>>()?;
    let first_ranks = first_words
        .iter()
        .map(|word| word_rank(&ordered_alphabet, word).map(|rank| rank.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let cycle_source = first_words
        .iter()
        .find(|word| word.windows(2).any(|window| window[0] != window[1]))
        .or_else(|| first_words.first());
    let canonical_cycles = cycle_source
        .map(|word| canonical_cycles(word))
        .unwrap_or_else(Vec::new);
    let longest_words = longest_only(first_words.clone(), |word| word.len());

    Ok(FiniteEnumerationDemo {
        alphabet,
        ordered_alphabet,
        length,
        limit,
        total_words,
        first_words,
        first_digits,
        first_ranks,
        canonical_cycles,
        longest_words,
    })
}

fn ordered_alphabet(alphabet: &[String], order: &[usize]) -> Result<Vec<String>, CombError> {
    if order.len() != alphabet.len() {
        return Err(CombError::InvalidParameters(
            "order length must match alphabet length".to_string(),
        ));
    }
    let mut seen = vec![false; alphabet.len()];
    let mut ordered = Vec::with_capacity(alphabet.len());
    for &index in order {
        if index >= alphabet.len() {
            return Err(CombError::OutOfRange {
                value: index.to_string(),
                bound: alphabet.len().to_string(),
            });
        }
        if seen[index] {
            return Err(CombError::InvalidParameters(
                "order must be a permutation".to_string(),
            ));
        }
        seen[index] = true;
        ordered.push(alphabet[index].clone());
    }
    Ok(ordered)
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

    #[test]
    fn finite_enumeration_demo_uses_supplied_data() {
        let demo = finite_enumeration_demo(
            vec!["C".to_string(), "D".to_string(), "E".to_string()],
            2,
            vec![2, 0, 1],
            5,
        )
        .expect("valid finite enumeration demo");

        assert_eq!(demo.ordered_alphabet, vec!["E", "C", "D"]);
        assert_eq!(demo.total_words, "9");
        assert_eq!(
            demo.first_words,
            vec![
                vec!["E", "E"],
                vec!["E", "C"],
                vec!["E", "D"],
                vec!["C", "E"],
                vec!["C", "C"],
            ]
        );
        assert_eq!(
            demo.first_digits,
            vec![vec![0, 0], vec![0, 1], vec![0, 2], vec![1, 0], vec![1, 1]]
        );
        assert_eq!(demo.first_ranks, vec!["0", "1", "2", "3", "4"]);
        assert_eq!(
            demo.canonical_cycles,
            vec![
                vec!["C".to_string(), "E".to_string()],
                vec!["E".to_string(), "C".to_string()]
            ]
        );
        assert_eq!(demo.longest_words, demo.first_words);
    }
}
