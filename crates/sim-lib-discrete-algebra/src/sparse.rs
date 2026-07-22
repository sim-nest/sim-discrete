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

    /// Checked empty `rows x cols` sparse matrix.
    pub fn try_new(rows: usize, cols: usize) -> Result<Self, AlgebraError> {
        let m = SparseMatrix {
            rows,
            cols,
            entries: Vec::new(),
        };
        m.validate()?;
        Ok(m)
    }

    /// Number of sparse matrix rows.
    pub fn row_count(&self) -> usize {
        self.rows
    }

    /// Number of sparse matrix columns.
    pub fn col_count(&self) -> usize {
        self.cols
    }

    /// Read-only sparse entries.
    pub fn entries(&self) -> &[SparseEntry<S>] {
        &self.entries
    }

    /// Build from entries, rejecting any whose coordinates are out of range.
    pub fn from_entries(
        rows: usize,
        cols: usize,
        entries: Vec<SparseEntry<S>>,
    ) -> Result<Self, AlgebraError> {
        let mut m = SparseMatrix {
            rows,
            cols,
            entries,
        };
        m.validate()?;
        m.canonicalize();
        Ok(m)
    }

    /// Validate public sparse dimensions and entry coordinates.
    pub fn validate(&self) -> Result<(), AlgebraError> {
        let len = self
            .rows
            .checked_mul(self.cols)
            .ok_or(AlgebraError::DimensionOverflow {
                rows: self.rows,
                cols: self.cols,
            })?;
        for e in &self.entries {
            if e.row >= self.rows || e.col >= self.cols {
                return Err(AlgebraError::IndexOutOfBounds {
                    index: e.row.saturating_mul(self.cols).saturating_add(e.col),
                    len,
                });
            }
        }
        Ok(())
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
    pub fn to_dense(&self) -> Result<Matrix<S>, AlgebraError> {
        self.validate()?;
        let mut m = Matrix::try_new(self.rows, self.cols)?;
        for e in &self.entries {
            m.set(e.row, e.col, e.value.clone())?;
        }
        Ok(m)
    }

    /// Build a canonical sparse matrix from a dense one, keeping non-`zero`
    /// entries only.
    pub fn from_dense_nonzero(dense: &Matrix<S>) -> Result<Self, AlgebraError> {
        dense.validate()?;
        let mut entries = Vec::new();
        for r in 0..dense.rows {
            for c in 0..dense.cols {
                let v = dense.get(r, c)?;
                if !v.is_zero() {
                    entries.push(SparseEntry {
                        row: r,
                        col: c,
                        value: v.clone(),
                    });
                }
            }
        }
        Ok(SparseMatrix {
            rows: dense.rows,
            cols: dense.cols,
            entries,
        })
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
        let dense = m.to_dense().unwrap();
        let back = SparseMatrix::from_dense_nonzero(&dense).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn invalid_public_sparse_matrix_fails_before_densifying() {
        let bad = SparseMatrix {
            rows: 1,
            cols: 1,
            entries: vec![e(1, 0, 4)],
        };

        assert!(matches!(
            bad.validate(),
            Err(AlgebraError::IndexOutOfBounds { .. })
        ));
        assert!(matches!(
            bad.to_dense(),
            Err(AlgebraError::IndexOutOfBounds { .. })
        ));
    }

    #[test]
    fn checked_sparse_constructor_rejects_dimension_overflow() {
        assert!(matches!(
            SparseMatrix::<Counting>::try_new(usize::MAX, 2),
            Err(AlgebraError::DimensionOverflow { .. })
        ));
    }
}
