//! Constrained fixed-alphabet word search fixture.

use std::collections::BTreeSet;

use crate::{SearchError, SearchProblem, SearchStep};

/// Search state for the constrained word problem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordSearchState {
    /// Prefix chosen so far.
    pub prefix: Vec<String>,
    /// Accumulated minimization score.
    pub score: i64,
}

/// Finished constrained word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordSearchSolution {
    /// Complete word.
    pub word: Vec<String>,
    /// Minimization score used by branch-and-bound and A-star policy.
    pub score: i64,
}

/// Deterministic fixed-alphabet word problem used by Rust and Lisp specimens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstrainedWordProblem {
    alphabet: Vec<String>,
    length: usize,
    required_first: String,
    required_last: String,
    forbidden_pairs: BTreeSet<(String, String)>,
    costly_symbol: Option<String>,
}

impl ConstrainedWordProblem {
    /// Build the default constrained word fixture.
    pub fn new(
        alphabet: Vec<String>,
        length: usize,
        required_first: String,
        required_last: String,
    ) -> Result<Self, SearchError> {
        if alphabet.is_empty() {
            return Err(SearchError::InvalidProblem(
                "alphabet must not be empty".to_string(),
            ));
        }
        if length < 2 {
            return Err(SearchError::InvalidProblem(
                "word length must be at least two".to_string(),
            ));
        }
        let unique = alphabet.iter().collect::<BTreeSet<_>>();
        if unique.len() != alphabet.len() {
            return Err(SearchError::InvalidProblem(
                "alphabet entries must be unique".to_string(),
            ));
        }
        if !unique.contains(&required_first) || !unique.contains(&required_last) {
            return Err(SearchError::InvalidProblem(
                "required endpoints must be in the alphabet".to_string(),
            ));
        }
        let costly_symbol = alphabet.get(1).cloned();
        let mut forbidden_pairs = BTreeSet::new();
        if alphabet.len() >= 3 {
            forbidden_pairs.insert((required_last.clone(), alphabet[1].clone()));
        }
        Ok(Self {
            alphabet,
            length,
            required_first,
            required_last,
            forbidden_pairs,
            costly_symbol,
        })
    }

    /// Add one forbidden adjacent pair.
    pub fn with_forbidden_pair(mut self, left: String, right: String) -> Self {
        self.forbidden_pairs.insert((left, right));
        self
    }

    /// Return the alphabet in deterministic choice order.
    pub fn alphabet(&self) -> &[String] {
        &self.alphabet
    }

    /// Return the target word length.
    pub fn length(&self) -> usize {
        self.length
    }
}

impl SearchProblem for ConstrainedWordProblem {
    type State = WordSearchState;
    type Choice = String;
    type Output = WordSearchSolution;

    fn initial_state(&self) -> Self::State {
        WordSearchState {
            prefix: Vec::new(),
            score: 0,
        }
    }

    fn expand(&self, state: &Self::State, out: &mut Vec<Self::Choice>) {
        if state.prefix.len() < self.length {
            out.extend(self.alphabet.iter().cloned());
        }
    }

    fn apply(&self, state: &Self::State, choice: &Self::Choice) -> SearchStep<Self::State> {
        let position = state.prefix.len();
        if position == 0 && choice != &self.required_first {
            return SearchStep::pruned("prefix does not match required first symbol");
        }
        if position + 1 == self.length && choice != &self.required_last {
            return SearchStep::pruned("suffix does not match required last symbol");
        }
        if state.prefix.last() == Some(choice) {
            return SearchStep::pruned("adjacent symbols must differ");
        }
        if let Some(previous) = state.prefix.last()
            && self
                .forbidden_pairs
                .contains(&(previous.clone(), choice.clone()))
        {
            return SearchStep::pruned("forbidden adjacent pair");
        }

        let mut prefix = state.prefix.clone();
        prefix.push(choice.clone());
        let score = state.score + i64::from(self.costly_symbol.as_ref() == Some(choice));
        SearchStep::Continue(WordSearchState { prefix, score })
    }

    fn propagate(&self, state: Self::State) -> SearchStep<Self::State> {
        if state.prefix.len() > self.length {
            return SearchStep::infeasible("prefix exceeds target length");
        }
        let remaining = self.length - state.prefix.len();
        if remaining == 1 && state.prefix.last() == Some(&self.required_last) {
            return SearchStep::pruned("required suffix would repeat");
        }
        SearchStep::Continue(state)
    }

    fn finish(&self, state: &Self::State) -> Option<Self::Output> {
        (state.prefix.len() == self.length && state.prefix.last() == Some(&self.required_last))
            .then(|| WordSearchSolution {
                word: state.prefix.clone(),
                score: state.score,
            })
    }

    fn score_state(&self, state: &Self::State) -> i64 {
        state.score
    }

    fn estimate_remaining(&self, _state: &Self::State) -> i64 {
        0
    }

    fn bound(&self, state: &Self::State) -> Option<i64> {
        Some(state.score)
    }

    fn output_score(&self, output: &Self::Output) -> Option<i64> {
        Some(output.score)
    }
}

/// Render the constrained word specimen in a stable line format.
pub fn render_constrained_word_demo(
    solutions: &[WordSearchSolution],
    receipt_digest: &str,
) -> String {
    let mut out = String::new();
    for solution in solutions {
        out.push_str(&solution.word.join(""));
        out.push('\t');
        out.push_str(&solution.score.to_string());
        out.push('\n');
    }
    out.push_str("receipt\t");
    out.push_str(receipt_digest);
    out.push('\n');
    out
}
