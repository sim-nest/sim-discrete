//! Fixed-alphabet words, cyclic patterns, and longest-only selection.
//!
//! Words are produced lazily in mixed-radix lexicographic order. The first
//! position is most significant, so rank `0` is the all-zero digit word and the
//! last position changes fastest.

// conformance: finite enumeration adapters preserve rank/unrank and lazy limits.

use crate::{CombError, mixed_radix_rank, mixed_radix_unrank};
use num_bigint::BigUint;

/// Iterator over fixed-length words drawn from one alphabet.
#[derive(Debug, Clone)]
pub struct MixedRadixWords<'a, T> {
    alphabet: &'a [T],
    digits: Option<Vec<usize>>,
    emitted: BigUint,
    total: BigUint,
}

impl<'a, T> MixedRadixWords<'a, T> {
    fn new(alphabet: &'a [T], length: usize) -> Self {
        let total = word_count(alphabet.len(), length);
        let digits = if length == 0 {
            Some(Vec::new())
        } else if alphabet.is_empty() {
            None
        } else {
            Some(vec![0; length])
        };
        Self {
            alphabet,
            digits,
            emitted: BigUint::from(0u32),
            total,
        }
    }

    /// Total number of words in this finite iterator.
    pub fn total_ordinals(&self) -> &BigUint {
        &self.total
    }

    /// Number of words not yet emitted.
    pub fn remaining_ordinals(&self) -> BigUint {
        if self.emitted >= self.total {
            BigUint::from(0u32)
        } else {
            &self.total - &self.emitted
        }
    }
}

impl<T: Clone> Iterator for MixedRadixWords<'_, T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let digits = self.digits.as_ref()?.clone();
        let word = digits
            .iter()
            .map(|&digit| self.alphabet[digit].clone())
            .collect();
        self.emitted += 1u32;

        let mut next_digits = digits;
        self.digits = if advance_digits(&mut next_digits, self.alphabet.len()) {
            Some(next_digits)
        } else {
            None
        };
        Some(word)
    }
}

/// Construct a lazy iterator over all fixed-length words from `alphabet`.
///
/// # Examples
///
/// ```
/// use sim_lib_discrete_comb::words;
///
/// let alphabet = ["A", "B"];
/// let generated: Vec<_> = words(&alphabet, 2).collect();
/// assert_eq!(
///     generated,
///     vec![vec!["A", "A"], vec!["A", "B"], vec!["B", "A"], vec!["B", "B"]]
/// );
/// ```
pub fn words<T: Clone>(alphabet: &[T], length: usize) -> MixedRadixWords<'_, T> {
    MixedRadixWords::new(alphabet, length)
}

/// Exact fixed-alphabet word count, `alphabet_len.pow(length)`.
pub fn word_count(alphabet_len: usize, length: usize) -> BigUint {
    if length == 0 {
        return BigUint::from(1u32);
    }
    if alphabet_len == 0 {
        return BigUint::from(0u32);
    }
    let radix = BigUint::from(alphabet_len);
    let mut total = BigUint::from(1u32);
    for _ in 0..length {
        total *= &radix;
    }
    total
}

/// Build the repeated mixed-radix vector for words over an alphabet.
pub fn word_radices(alphabet_len: usize, length: usize) -> Result<Vec<u64>, CombError> {
    if length == 0 {
        return Ok(Vec::new());
    }
    if alphabet_len == 0 {
        return Err(CombError::InvalidParameters(
            "word radices require a non-empty alphabet for non-empty words".to_string(),
        ));
    }
    let radix = u64::try_from(alphabet_len).map_err(|_| {
        CombError::LimitExceeded(format!("alphabet length {alphabet_len} exceeds u64"))
    })?;
    Ok(vec![radix; length])
}

/// Convert mixed-radix digits into a word over `alphabet`.
pub fn digits_to_word<T: Clone>(alphabet: &[T], digits: &[u64]) -> Result<Vec<T>, CombError> {
    digits
        .iter()
        .map(|&digit| {
            let index = usize::try_from(digit).map_err(|_| CombError::OutOfRange {
                value: digit.to_string(),
                bound: alphabet.len().to_string(),
            })?;
            alphabet.get(index).cloned().ok_or(CombError::OutOfRange {
                value: digit.to_string(),
                bound: alphabet.len().to_string(),
            })
        })
        .collect()
}

/// Convert a word into mixed-radix digits over a unique `alphabet`.
pub fn word_to_digits<T: Eq>(alphabet: &[T], word: &[T]) -> Result<Vec<u64>, CombError> {
    reject_duplicate_alphabet(alphabet)?;
    word.iter()
        .map(|item| {
            alphabet
                .iter()
                .position(|candidate| candidate == item)
                .map(|index| {
                    u64::try_from(index).map_err(|_| {
                        CombError::LimitExceeded(format!("word index {index} exceeds u64"))
                    })
                })
                .transpose()?
                .ok_or_else(|| CombError::InvalidParameters("word item is not in alphabet".into()))
        })
        .collect()
}

/// Rank a word using the fixed-alphabet mixed-radix order.
pub fn word_rank<T: Eq>(alphabet: &[T], word: &[T]) -> Result<BigUint, CombError> {
    let digits = word_to_digits(alphabet, word)?;
    let radices = word_radices(alphabet.len(), word.len())?;
    mixed_radix_rank(&digits, &radices)
}

/// Unrank a fixed-length word from the fixed-alphabet mixed-radix order.
pub fn word_unrank<T: Clone>(
    alphabet: &[T],
    length: usize,
    rank: &BigUint,
) -> Result<Vec<T>, CombError> {
    let total = word_count(alphabet.len(), length);
    if rank >= &total {
        return Err(CombError::OutOfRange {
            value: rank.to_string(),
            bound: total.to_string(),
        });
    }
    let radices = word_radices(alphabet.len(), length)?;
    let digits = mixed_radix_unrank(rank, &radices)?;
    digits_to_word(alphabet, &digits)
}

/// Return every unique cyclic rotation in canonical sorted order.
///
/// The first returned word is the canonical representative for the rotation
/// class.
pub fn canonical_cycles<T: Ord + Clone>(word: &[T]) -> Vec<Vec<T>> {
    if word.is_empty() {
        return vec![Vec::new()];
    }
    let mut rotations = (0..word.len())
        .map(|start| {
            word[start..]
                .iter()
                .chain(&word[..start])
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    rotations.sort();
    rotations.dedup();
    rotations
}

/// Keep only the items whose measured length is maximal.
pub fn longest_only<T>(items: impl IntoIterator<Item = T>, len: impl Fn(&T) -> usize) -> Vec<T> {
    let mut longest = Vec::new();
    let mut best = None;
    for item in items {
        let item_len = len(&item);
        match best {
            None => {
                best = Some(item_len);
                longest.push(item);
            }
            Some(current) if item_len > current => {
                best = Some(item_len);
                longest.clear();
                longest.push(item);
            }
            Some(current) if item_len == current => longest.push(item),
            Some(_) => {}
        }
    }
    longest
}

fn advance_digits(digits: &mut [usize], radix: usize) -> bool {
    for index in (0..digits.len()).rev() {
        digits[index] += 1;
        if digits[index] < radix {
            return true;
        }
        digits[index] = 0;
    }
    false
}

fn reject_duplicate_alphabet<T: Eq>(alphabet: &[T]) -> Result<(), CombError> {
    for left in 0..alphabet.len() {
        for right in (left + 1)..alphabet.len() {
            if alphabet[left] == alphabet[right] {
                return Err(CombError::InvalidParameters(
                    "alphabet contains duplicate values".to_string(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Debug)]
    struct CountedClone {
        value: u8,
        clones: Arc<AtomicUsize>,
    }

    impl Clone for CountedClone {
        fn clone(&self) -> Self {
            self.clones.fetch_add(1, Ordering::SeqCst);
            Self {
                value: self.value,
                clones: Arc::clone(&self.clones),
            }
        }
    }

    #[test]
    fn words_are_lazy_and_lexicographic() {
        let clones = Arc::new(AtomicUsize::new(0));
        let alphabet = [
            CountedClone {
                value: 0,
                clones: Arc::clone(&clones),
            },
            CountedClone {
                value: 1,
                clones: Arc::clone(&clones),
            },
        ];

        let first: Vec<_> = words(&alphabet, 8).take(3).collect();
        assert_eq!(
            first
                .iter()
                .map(|word| word.iter().map(|item| item.value).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![
                vec![0, 0, 0, 0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0, 0, 0, 1],
                vec![0, 0, 0, 0, 0, 0, 1, 0],
            ]
        );
        assert_eq!(clones.load(Ordering::SeqCst), 24);
    }

    #[test]
    fn exact_count_handles_large_spaces() {
        assert_eq!(word_count(3, 5), BigUint::from(243u32));
        assert_eq!(word_count(2, 130), BigUint::from(1u32) << 130usize);
        assert_eq!(words::<u8>(&[], 0).count(), 1);
        assert_eq!(words::<u8>(&[], 3).count(), 0);
    }

    #[test]
    fn word_digits_rank_and_unrank_round_trip() {
        let alphabet = ["A", "B", "C"];
        for (ordinal, word) in words(&alphabet, 3).enumerate() {
            let rank = word_rank(&alphabet, &word).unwrap();
            assert_eq!(rank, BigUint::from(ordinal as u32));
            assert_eq!(word_unrank(&alphabet, 3, &rank).unwrap(), word);
        }
        assert_eq!(
            digits_to_word(&alphabet, &[2, 0, 1]).unwrap(),
            vec!["C", "A", "B"]
        );
        assert_eq!(
            word_to_digits(&alphabet, &["C", "A", "B"]).unwrap(),
            vec![2, 0, 1]
        );
    }

    #[test]
    fn adapters_reject_invalid_word_domains() {
        assert!(matches!(
            word_rank(&["A", "A"], &["A"]),
            Err(CombError::InvalidParameters(_))
        ));
        assert!(matches!(
            word_rank(&["A"], &["B"]),
            Err(CombError::InvalidParameters(_))
        ));
        assert!(matches!(
            word_unrank::<&str>(&[], 2, &BigUint::from(0u32)),
            Err(CombError::OutOfRange { .. })
        ));
    }

    #[test]
    fn cycles_are_unique_and_canonical() {
        assert_eq!(canonical_cycles::<u8>(&[]), vec![Vec::<u8>::new()]);
        assert_eq!(
            canonical_cycles(&[2, 1, 2, 1]),
            vec![vec![1, 2, 1, 2], vec![2, 1, 2, 1]]
        );
        assert_eq!(
            canonical_cycles(&[3, 1, 2]),
            vec![vec![1, 2, 3], vec![2, 3, 1], vec![3, 1, 2]]
        );
    }

    #[test]
    fn longest_only_keeps_ties_without_materializing_losers() {
        let longest = longest_only(vec!["a", "abcd", "xy", "wxyz"], |item| item.len());
        assert_eq!(longest, vec!["abcd", "wxyz"]);
    }
}
