//! Coordinate-list sparse matrix over a [`Semiring`].
//!
//! v1 uses the simple, codec-friendly coordinate-list (COO) form. Duplicate
//! coordinates are merged by semiring `add` during `canonicalize`, which also
//! drops `zero` entries and sorts by `(row, col)`. CSR/CSC may be added later as
//! internal acceleration without changing this public contract.

use crate::error::AlgebraError;
use crate::matrix::Matrix;
use crate::semiring::Semiring;

/// A single non-default entry of a sparse matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseEntry<S> {
    /// Row index.
    pub row: usize,
    /// Column index.
    pub col: usize,
    /// Stored value.
    pub value: S,
}

/// A coordinate-list sparse matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct SparseMatrix<S: Semiring> {
    /// Number of rows.
    pub rows: usize,
    /// Number of columns.
    pub cols: usize,
    /// Stored entries (canonical after [`SparseMatrix::canonicalize`]).
    pub entries: Vec<SparseEntry<S>>,
}

impl<S: Semiring> SparseMatrix<S> {
    /// An empty `rows x cols` sparse matrix.
    pub fn new(rows: usize, cols: usize) -> Self {
        SparseMatrix {
            rows,
            cols,
            entries: Vec::new(),
        }
    }

    /// Build from entries, rejecting any whose coordinates are out of range.
    pub fn from_entries(
        rows: usize,
        cols: usize,
        entries: Vec<SparseEntry<S>>,
    ) -> Result<Self, AlgebraError> {
        for e in &entries {
            if e.row >= rows || e.col >= cols {
                return Err(AlgebraError::IndexOutOfBounds {
                    index: e.row.saturating_mul(cols).saturating_add(e.col),
                    len: rows.saturating_mul(cols),
                });
            }
        }
        let mut m = SparseMatrix {
            rows,
            cols,
            entries,
        };
        m.canonicalize();
        Ok(m)
    }

    /// Sort entries by `(row, col)`, merge duplicates by semiring `add`, and
    /// drop entries equal to `zero`.
    pub fn canonicalize(&mut self) {
        self.entries.sort_by_key(|e| (e.row, e.col));
        let mut merged: Vec<SparseEntry<S>> = Vec::with_capacity(self.entries.len());
        for e in self.entries.drain(..) {
            match merged.last_mut() {
                Some(last) if last.row == e.row && last.col == e.col => {
                    last.value = last.value.add(&e.value);
                }
                _ => merged.push(e),
            }
        }
        merged.retain(|e| !e.value.is_zero());
        self.entries = merged;
    }

    /// Densify, filling absent positions with the semiring `zero`.
    pub fn to_dense(&self) -> Matrix<S> {
        let mut m = Matrix::new(self.rows, self.cols);
        for e in &self.entries {
            m.data[e.row * self.cols + e.col] = e.value.clone();
        }
        m
    }

    /// Build a canonical sparse matrix from a dense one, keeping non-`zero`
    /// entries only.
    pub fn from_dense_nonzero(dense: &Matrix<S>) -> Self {
        let mut entries = Vec::new();
        for r in 0..dense.rows {
            for c in 0..dense.cols {
                let v = &dense.data[r * dense.cols + c];
                if !v.is_zero() {
                    entries.push(SparseEntry {
                        row: r,
                        col: c,
                        value: v.clone(),
                    });
                }
            }
        }
        SparseMatrix {
            rows: dense.rows,
            cols: dense.cols,
            entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Counting;

    fn e(row: usize, col: usize, v: u64) -> SparseEntry<Counting> {
        SparseEntry {
            row,
            col,
            value: Counting::from_u64(v),
        }
    }

    #[test]
    fn from_entries_rejects_out_of_range() {
        let r = SparseMatrix::from_entries(2, 2, vec![e(0, 0, 1), e(2, 0, 1)]);
        assert!(matches!(r, Err(AlgebraError::IndexOutOfBounds { .. })));
    }

    #[test]
    fn canonicalize_merges_duplicates_and_sorts() {
        let m = SparseMatrix::from_entries(2, 2, vec![e(1, 1, 2), e(0, 0, 3), e(1, 1, 5)]).unwrap();
        assert_eq!(m.entries, vec![e(0, 0, 3), e(1, 1, 7)]);
    }

    #[test]
    fn canonicalize_drops_zero() {
        // A merge that produces zero in the counting semiring only happens when
        // both are zero; an explicit zero entry is dropped.
        let m = SparseMatrix::from_entries(1, 2, vec![e(0, 0, 0), e(0, 1, 4)]).unwrap();
        assert_eq!(m.entries, vec![e(0, 1, 4)]);
    }

    #[test]
    fn dense_sparse_round_trip() {
        let m = SparseMatrix::from_entries(2, 2, vec![e(0, 1, 7), e(1, 0, 9)]).unwrap();
        let dense = m.to_dense();
        let back = SparseMatrix::from_dense_nonzero(&dense);
        assert_eq!(back, m);
    }
}
