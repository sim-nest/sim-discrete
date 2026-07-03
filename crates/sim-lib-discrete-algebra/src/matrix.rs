//! Generic dense matrix over a [`Semiring`], with semiring matrix multiply.
//!
//! Row-major storage: element `(r, c)` lives at `data[r * cols + c]`. The
//! `data.len() == rows * cols` invariant is maintained by every constructor;
//! the fields are public so sibling modules (`power`, `closure`) and the graph
//! crate can build matrices directly, but external mutators must keep it.

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

    /// A `rows x cols` matrix filled with `value`.
    pub fn filled(rows: usize, cols: usize, value: S) -> Self {
        Matrix {
            rows,
            cols,
            data: vec![value; rows * cols],
        }
    }

    /// The `n x n` identity: `one` on the diagonal, `zero` elsewhere.
    pub fn identity(n: usize) -> Self {
        let mut m = Self::new(n, n);
        for i in 0..n {
            m.data[i * n + i] = S::one();
        }
        m
    }

    /// Build from a vector of rows, rejecting ragged input.
    pub fn from_rows(rows: Vec<Vec<S>>) -> Result<Self, AlgebraError> {
        let nrows = rows.len();
        let ncols = rows.first().map_or(0, Vec::len);
        let mut data = Vec::with_capacity(nrows * ncols);
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

    /// Bounds-checked read of entry `(r, c)`.
    pub fn get(&self, r: usize, c: usize) -> Result<&S, AlgebraError> {
        if r >= self.rows || c >= self.cols {
            return Err(AlgebraError::IndexOutOfBounds {
                index: r.saturating_mul(self.cols).saturating_add(c),
                len: self.data.len(),
            });
        }
        Ok(&self.data[r * self.cols + c])
    }

    /// Bounds-checked write of entry `(r, c)`.
    pub fn set(&mut self, r: usize, c: usize, value: S) -> Result<(), AlgebraError> {
        if r >= self.rows || c >= self.cols {
            return Err(AlgebraError::IndexOutOfBounds {
                index: r.saturating_mul(self.cols).saturating_add(c),
                len: self.data.len(),
            });
        }
        self.data[r * self.cols + c] = value;
        Ok(())
    }

    /// Immutable slice of row `r`, or an error if out of range.
    pub fn row(&self, r: usize) -> Result<&[S], AlgebraError> {
        if r >= self.rows {
            return Err(AlgebraError::IndexOutOfBounds {
                index: r,
                len: self.rows,
            });
        }
        Ok(&self.data[r * self.cols..(r + 1) * self.cols])
    }

    /// The transpose (a fresh `cols x rows` matrix).
    pub fn transpose(&self) -> Self {
        let mut data = Vec::with_capacity(self.data.len());
        for c in 0..self.cols {
            for r in 0..self.rows {
                data.push(self.data[r * self.cols + c].clone());
            }
        }
        Matrix {
            rows: self.cols,
            cols: self.rows,
            data,
        }
    }

    /// Semiring matrix multiply: `self` (`m x p`) by `other` (`p x q`).
    pub fn matmul(&self, other: &Self) -> Result<Self, AlgebraError> {
        if self.cols != other.rows {
            return Err(AlgebraError::ShapeMismatch(format!(
                "matmul: {}x{} by {}x{}",
                self.rows, self.cols, other.rows, other.cols
            )));
        }
        let mut data = Vec::with_capacity(self.rows * other.cols);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut acc = S::zero();
                for k in 0..self.cols {
                    let term = self.data[i * self.cols + k].mul(&other.data[k * other.cols + j]);
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
