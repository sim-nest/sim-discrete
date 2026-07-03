//! Implicit matrix views that act without materializing dense storage.
//!
//! Each view provides a `matvec` (apply to a vector) and an explicit
//! `materialize` guarded by [`AlgebraLimits`], so a large view never silently
//! allocates a dense matrix. The spectral crate's Hadamard view reuses this
//! contract.

use crate::error::AlgebraError;
use crate::matrix::{AlgebraLimits, Matrix};
use crate::semiring::Semiring;

fn check_len<S: Semiring>(view_dim: usize, v: &[S]) -> Result<(), AlgebraError> {
    if v.len() != view_dim {
        return Err(AlgebraError::ShapeMismatch(format!(
            "matvec: vector len {} != view dimension {view_dim}",
            v.len()
        )));
    }
    Ok(())
}

fn check_dim(n: usize, limits: AlgebraLimits) -> Result<(), AlgebraError> {
    if n > limits.max_dim {
        return Err(AlgebraError::LimitExceeded(format!(
            "materialize: dimension {n} exceeds max_dim {}",
            limits.max_dim
        )));
    }
    Ok(())
}

/// The `n x n` identity, applied without storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentityView {
    /// The dimension `n`.
    pub n: usize,
}

impl IdentityView {
    /// Apply to a vector: returns a clone of `v`.
    pub fn matvec<S: Semiring>(&self, v: &[S]) -> Result<Vec<S>, AlgebraError> {
        check_len(self.n, v)?;
        Ok(v.to_vec())
    }

    /// Materialize as a dense identity, guarded by `limits`.
    pub fn materialize<S: Semiring>(
        &self,
        limits: AlgebraLimits,
    ) -> Result<Matrix<S>, AlgebraError> {
        check_dim(self.n, limits)?;
        Ok(Matrix::identity(self.n))
    }
}

/// A diagonal matrix view with entries `diag`.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagonalView<S: Semiring> {
    /// Diagonal entries; the view is `diag.len() x diag.len()`.
    pub diag: Vec<S>,
}

impl<S: Semiring> DiagonalView<S> {
    /// Apply to a vector: `result[i] = diag[i] * v[i]`.
    pub fn matvec(&self, v: &[S]) -> Result<Vec<S>, AlgebraError> {
        check_len(self.diag.len(), v)?;
        Ok(self
            .diag
            .iter()
            .zip(v.iter())
            .map(|(d, x)| d.mul(x))
            .collect())
    }

    /// Materialize as a dense diagonal matrix, guarded by `limits`.
    pub fn materialize(&self, limits: AlgebraLimits) -> Result<Matrix<S>, AlgebraError> {
        let n = self.diag.len();
        check_dim(n, limits)?;
        let mut m = Matrix::new(n, n);
        for (i, d) in self.diag.iter().enumerate() {
            m.data[i * n + i] = d.clone();
        }
        Ok(m)
    }
}

/// A permutation matrix view: row `i` has its `one` in column `perm[i]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermutationView {
    /// `perm[i]` is the column holding the `one` in row `i`.
    pub perm: Vec<usize>,
}

impl PermutationView {
    /// Apply to a vector: `result[i] = v[perm[i]]`.
    pub fn matvec<S: Semiring>(&self, v: &[S]) -> Result<Vec<S>, AlgebraError> {
        check_len(self.perm.len(), v)?;
        let mut out = Vec::with_capacity(self.perm.len());
        for &p in &self.perm {
            let val = v
                .get(p)
                .ok_or(AlgebraError::IndexOutOfBounds {
                    index: p,
                    len: v.len(),
                })?
                .clone();
            out.push(val);
        }
        Ok(out)
    }

    /// Materialize as a dense permutation matrix, guarded by `limits`.
    pub fn materialize<S: Semiring>(
        &self,
        limits: AlgebraLimits,
    ) -> Result<Matrix<S>, AlgebraError> {
        let n = self.perm.len();
        check_dim(n, limits)?;
        let mut m: Matrix<S> = Matrix::new(n, n);
        for (i, &p) in self.perm.iter().enumerate() {
            if p >= n {
                return Err(AlgebraError::IndexOutOfBounds { index: p, len: n });
            }
            m.data[i * n + p] = S::one();
        }
        Ok(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Counting;

    fn vec_u64(xs: &[u64]) -> Vec<Counting> {
        xs.iter().map(|&x| Counting::from_u64(x)).collect()
    }

    #[test]
    fn identity_matvec_returns_input() {
        let v = vec_u64(&[3, 5, 7]);
        assert_eq!(IdentityView { n: 3 }.matvec(&v).unwrap(), v);
    }

    #[test]
    fn diagonal_matvec_scales() {
        let d = DiagonalView {
            diag: vec_u64(&[2, 3]),
        };
        assert_eq!(d.matvec(&vec_u64(&[5, 7])).unwrap(), vec_u64(&[10, 21]));
    }

    #[test]
    fn permutation_matvec_permutes() {
        // perm = [2,0,1] gathers v[2], v[0], v[1].
        let p = PermutationView {
            perm: vec![2, 0, 1],
        };
        assert_eq!(p.matvec(&vec_u64(&[1, 2, 3])).unwrap(), vec_u64(&[3, 1, 2]));
    }

    #[test]
    fn permutation_materialize_matches_matvec() {
        let p = PermutationView {
            perm: vec![2, 0, 1],
        };
        let m: Matrix<Counting> = p.materialize(AlgebraLimits::default()).unwrap();
        let v = vec_u64(&[1, 2, 3]);
        // Dense P * v equals the view's matvec.
        let dense_col = Matrix::from_rows(v.iter().map(|x| vec![x.clone()]).collect()).unwrap();
        let prod = m.matmul(&dense_col).unwrap();
        assert_eq!(prod.data, p.matvec(&v).unwrap());
    }

    #[test]
    fn materialize_respects_limit() {
        let id = IdentityView { n: 100 };
        let limited = AlgebraLimits { max_dim: 10 };
        assert!(matches!(
            id.materialize::<Counting>(limited),
            Err(AlgebraError::LimitExceeded(_))
        ));
    }
}
