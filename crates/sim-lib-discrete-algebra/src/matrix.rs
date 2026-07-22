//! Generic dense matrix over a [`Semiring`], with semiring matrix multiply.
//!
//! Row-major storage: element `(r, c)` lives at `data[r * cols + c]`. The
//! `data.len() == rows * cols` invariant is maintained by every checked
//! constructor. The fields remain public for wire compatibility, so every
//! fallible reader and operator validates public values before indexing.

use crate::error::AlgebraError;
use crate::semiring::Semiring;

/// Explicit limits for potentially expensive matrix operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlgebraLimits {
    /// Maximum allowed dimension `n` for `power` / `closure` / `materialize`.
    pub max_dim: usize,
}

impl Default for AlgebraLimits {
    fn default() -> Self {
        AlgebraLimits { max_dim: 1024 }
    }
}

impl AlgebraLimits {
    /// Effectively unlimited; intended for tests.
    pub fn unlimited() -> Self {
        AlgebraLimits {
            max_dim: usize::MAX,
        }
    }

    fn check_matrix_dims(&self, rows: usize, cols: usize, op: &str) -> Result<(), AlgebraError> {
        let max_dim = rows.max(cols);
        if max_dim > self.max_dim {
            return Err(AlgebraError::LimitExceeded(format!(
                "{op}: dimension {max_dim} exceeds max_dim {}",
                self.max_dim
            )));
        }
        Ok(())
    }
}

/// A dense, row-major matrix over the semiring `S`.
///
/// # Examples
///
/// Build matrices over the counting semiring and multiply them; the identity
/// acts as a multiplicative unit:
///
/// ```
/// use sim_lib_discrete_algebra::{Counting, Matrix};
///
/// let a = Matrix::from_rows(vec![
///     vec![Counting::from_u64(1), Counting::from_u64(2)],
///     vec![Counting::from_u64(3), Counting::from_u64(4)],
/// ])
/// .unwrap();
/// let id = Matrix::identity(2);
///
/// assert_eq!(a.matmul(&id).unwrap(), a);
/// assert_eq!(a.get(1, 0).unwrap(), &Counting::from_u64(3));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<S: Semiring> {
    /// Number of rows.
    pub rows: usize,
    /// Number of columns.
    pub cols: usize,
    /// Row-major entries; `data.len() == rows * cols`.
    pub data: Vec<S>,
}

impl<S: Semiring> Matrix<S> {
    /// A `rows x cols` matrix filled with the semiring `zero`.
    pub fn new(rows: usize, cols: usize) -> Self {
        Self::filled(rows, cols, S::zero())
    }

    /// Checked `rows x cols` matrix filled with the semiring `zero`.
    pub fn try_new(rows: usize, cols: usize) -> Result<Self, AlgebraError> {
        Self::try_filled(rows, cols, S::zero())
    }

    /// Checked `rows x cols` matrix filled with the semiring `zero`, guarded by
    /// an explicit dimension limit.
    pub fn try_new_with_limits(
        rows: usize,
        cols: usize,
        limits: AlgebraLimits,
    ) -> Result<Self, AlgebraError> {
        Self::try_filled_with_limits(rows, cols, S::zero(), limits)
    }

    /// A `rows x cols` matrix filled with `value`.
    pub fn filled(rows: usize, cols: usize, value: S) -> Self {
        let len = checked_len(rows, cols).expect("matrix dimensions must fit in usize");
        Matrix {
            rows,
            cols,
            data: vec![value; len],
        }
    }

    /// Checked `rows x cols` matrix filled with `value`.
    pub fn try_filled(rows: usize, cols: usize, value: S) -> Result<Self, AlgebraError> {
        let len = checked_len(rows, cols)?;
        Ok(Matrix {
            rows,
            cols,
            data: vec![value; len],
        })
    }

    /// Checked `rows x cols` matrix filled with `value`, guarded by an explicit
    /// dimension limit.
    pub fn try_filled_with_limits(
        rows: usize,
        cols: usize,
        value: S,
        limits: AlgebraLimits,
    ) -> Result<Self, AlgebraError> {
        limits.check_matrix_dims(rows, cols, "matrix construction")?;
        Self::try_filled(rows, cols, value)
    }

    /// The `n x n` identity: `one` on the diagonal, `zero` elsewhere.
    pub fn identity(n: usize) -> Self {
        let mut m = Self::new(n, n);
        for i in 0..n {
            m.data[i * n + i] = S::one();
        }
        m
    }

    /// Checked `n x n` identity matrix.
    pub fn try_identity(n: usize) -> Result<Self, AlgebraError> {
        let mut m = Self::try_new(n, n)?;
        for i in 0..n {
            m.data[i * n + i] = S::one();
        }
        Ok(m)
    }

    /// Checked `n x n` identity matrix guarded by an explicit dimension limit.
    pub fn try_identity_with_limits(n: usize, limits: AlgebraLimits) -> Result<Self, AlgebraError> {
        limits.check_matrix_dims(n, n, "identity construction")?;
        Self::try_identity(n)
    }

    /// Build from a vector of rows, rejecting ragged input.
    pub fn from_rows(rows: Vec<Vec<S>>) -> Result<Self, AlgebraError> {
        let nrows = rows.len();
        let ncols = rows.first().map_or(0, Vec::len);
        let expected = checked_len(nrows, ncols)?;
        let mut data = Vec::with_capacity(expected);
        for row in rows {
            if row.len() != ncols {
                return Err(AlgebraError::Ragged);
            }
            data.extend(row);
        }
        Ok(Matrix {
            rows: nrows,
            cols: ncols,
            data,
        })
    }

    /// Whether the matrix is square.
    pub fn is_square(&self) -> bool {
        self.rows == self.cols
    }

    /// Number of matrix rows.
    pub fn row_count(&self) -> usize {
        self.rows
    }

    /// Number of matrix columns.
    pub fn col_count(&self) -> usize {
        self.cols
    }

    /// Read-only row-major data slice.
    pub fn data(&self) -> &[S] {
        &self.data
    }

    /// Validate the public structural invariant before indexing by shape.
    pub fn validate(&self) -> Result<(), AlgebraError> {
        let expected = checked_len(self.rows, self.cols)?;
        if self.data.len() != expected {
            return Err(AlgebraError::InvalidMatrix {
                rows: self.rows,
                cols: self.cols,
                expected,
                actual: self.data.len(),
            });
        }
        Ok(())
    }

    /// Bounds-checked read of entry `(r, c)`.
    pub fn get(&self, r: usize, c: usize) -> Result<&S, AlgebraError> {
        self.validate()?;
        if r >= self.rows || c >= self.cols {
            return Err(AlgebraError::IndexOutOfBounds {
                index: r.saturating_mul(self.cols).saturating_add(c),
                len: self.data.len(),
            });
        }
        Ok(&self.data[offset(self.cols, r, c)?])
    }

    /// Bounds-checked write of entry `(r, c)`.
    pub fn set(&mut self, r: usize, c: usize, value: S) -> Result<(), AlgebraError> {
        self.validate()?;
        if r >= self.rows || c >= self.cols {
            return Err(AlgebraError::IndexOutOfBounds {
                index: r.saturating_mul(self.cols).saturating_add(c),
                len: self.data.len(),
            });
        }
        let index = offset(self.cols, r, c)?;
        self.data[index] = value;
        Ok(())
    }

    /// Immutable slice of row `r`, or an error if out of range.
    pub fn row(&self, r: usize) -> Result<&[S], AlgebraError> {
        self.validate()?;
        if r >= self.rows {
            return Err(AlgebraError::IndexOutOfBounds {
                index: r,
                len: self.rows,
            });
        }
        let start = r
            .checked_mul(self.cols)
            .ok_or(AlgebraError::DimensionOverflow {
                rows: r,
                cols: self.cols,
            })?;
        let end = start
            .checked_add(self.cols)
            .ok_or(AlgebraError::DimensionOverflow {
                rows: r + 1,
                cols: self.cols,
            })?;
        Ok(&self.data[start..end])
    }

    /// The transpose (a fresh `cols x rows` matrix).
    pub fn transpose(&self) -> Result<Self, AlgebraError> {
        self.validate()?;
        let len = checked_len(self.cols, self.rows)?;
        let mut data = Vec::with_capacity(len);
        for c in 0..self.cols {
            for r in 0..self.rows {
                data.push(self.data[offset(self.cols, r, c)?].clone());
            }
        }
        Ok(Matrix {
            rows: self.cols,
            cols: self.rows,
            data,
        })
    }

    /// Semiring matrix multiply: `self` (`m x p`) by `other` (`p x q`).
    pub fn matmul(&self, other: &Self) -> Result<Self, AlgebraError> {
        self.validate()?;
        other.validate()?;
        if self.cols != other.rows {
            return Err(AlgebraError::ShapeMismatch(format!(
                "matmul: {}x{} by {}x{}",
                self.rows, self.cols, other.rows, other.cols
            )));
        }
        let len = checked_len(self.rows, other.cols)?;
        let mut data = Vec::with_capacity(len);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut acc = S::zero();
                for k in 0..self.cols {
                    let term = self.data[offset(self.cols, i, k)?]
                        .mul(&other.data[offset(other.cols, k, j)?]);
                    acc = acc.add(&term);
                }
                data.push(acc);
            }
        }
        Ok(Matrix {
            rows: self.rows,
            cols: other.cols,
            data,
        })
    }
}

fn checked_len(rows: usize, cols: usize) -> Result<usize, AlgebraError> {
    rows.checked_mul(cols)
        .ok_or(AlgebraError::DimensionOverflow { rows, cols })
}

fn offset(cols: usize, row: usize, col: usize) -> Result<usize, AlgebraError> {
    row.checked_mul(cols)
        .and_then(|base| base.checked_add(col))
        .ok_or(AlgebraError::DimensionOverflow { rows: row, cols })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Counting;
    use crate::tropical_min::MinPlus;

    #[test]
    fn from_rows_rejects_ragged() {
        let r = Matrix::from_rows(vec![
            vec![Counting::from_u64(1)],
            vec![Counting::from_u64(1), Counting::from_u64(2)],
        ]);
        assert_eq!(r.unwrap_err(), AlgebraError::Ragged);
    }

    #[test]
    fn matmul_known_result_over_counting() {
        // [[1,2],[3,4]] * [[5,6],[7,8]] = [[19,22],[43,50]] over the integers.
        let a = Matrix::from_rows(vec![
            vec![Counting::from_u64(1), Counting::from_u64(2)],
            vec![Counting::from_u64(3), Counting::from_u64(4)],
        ])
        .unwrap();
        let b = Matrix::from_rows(vec![
            vec![Counting::from_u64(5), Counting::from_u64(6)],
            vec![Counting::from_u64(7), Counting::from_u64(8)],
        ])
        .unwrap();
        let c = a.matmul(&b).unwrap();
        assert_eq!(c.data[0], Counting::from_u64(19));
        assert_eq!(c.data[1], Counting::from_u64(22));
        assert_eq!(c.data[2], Counting::from_u64(43));
        assert_eq!(c.data[3], Counting::from_u64(50));
    }

    #[test]
    fn matmul_shape_mismatch() {
        let a: Matrix<MinPlus> = Matrix::new(2, 3);
        let b: Matrix<MinPlus> = Matrix::new(2, 2);
        assert!(matches!(a.matmul(&b), Err(AlgebraError::ShapeMismatch(_))));
    }

    #[test]
    fn invalid_public_matrix_fails_before_indexing() {
        let bad = Matrix {
            rows: 2,
            cols: 2,
            data: vec![Counting::from_u64(1), Counting::from_u64(2)],
        };

        assert!(matches!(
            bad.validate(),
            Err(AlgebraError::InvalidMatrix { .. })
        ));
        assert!(matches!(
            bad.get(0, 0),
            Err(AlgebraError::InvalidMatrix { .. })
        ));
        assert!(matches!(
            bad.row(0),
            Err(AlgebraError::InvalidMatrix { .. })
        ));
        assert!(matches!(
            bad.transpose(),
            Err(AlgebraError::InvalidMatrix { .. })
        ));
        assert!(matches!(
            bad.matmul(&Matrix::identity(2)),
            Err(AlgebraError::InvalidMatrix { .. })
        ));
    }

    #[test]
    fn checked_constructors_reject_dimension_overflow() {
        assert!(matches!(
            Matrix::<Counting>::try_new(usize::MAX, 2),
            Err(AlgebraError::DimensionOverflow { .. })
        ));
        assert!(matches!(
            Matrix::<Counting>::try_filled(2, usize::MAX, Counting::from_u64(1)),
            Err(AlgebraError::DimensionOverflow { .. })
        ));
    }

    #[test]
    fn identity_is_multiplicative_unit() {
        let a = Matrix::from_rows(vec![
            vec![Counting::from_u64(1), Counting::from_u64(2)],
            vec![Counting::from_u64(3), Counting::from_u64(4)],
        ])
        .unwrap();
        let id = Matrix::identity(2);
        assert_eq!(a.matmul(&id).unwrap(), a);
        assert_eq!(id.matmul(&a).unwrap(), a);
    }
}
