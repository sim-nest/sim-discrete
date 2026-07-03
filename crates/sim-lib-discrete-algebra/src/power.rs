//! Matrix power by square-and-multiply. `A^k[i][j]` over the counting semiring
//! counts walks of length `k`; over min-plus it bounds `k`-edge shortest paths.

use crate::error::AlgebraError;
use crate::matrix::{AlgebraLimits, Matrix};
use crate::semiring::Semiring;

impl<S: Semiring> Matrix<S> {
    /// Raise a square matrix to the `k`-th power over its semiring.
    ///
    /// `k == 0` returns the identity. Returns [`AlgebraError::ShapeMismatch`]
    /// for non-square input and [`AlgebraError::LimitExceeded`] when the
    /// dimension exceeds `limits.max_dim`.
    pub fn power(&self, k: usize, limits: AlgebraLimits) -> Result<Self, AlgebraError> {
        if !self.is_square() {
            return Err(AlgebraError::ShapeMismatch(format!(
                "power requires a square matrix, got {}x{}",
                self.rows, self.cols
            )));
        }
        if self.rows > limits.max_dim {
            return Err(AlgebraError::LimitExceeded(format!(
                "power: dimension {} exceeds max_dim {}",
                self.rows, limits.max_dim
            )));
        }
        let n = self.rows;
        let mut result = Matrix::identity(n);
        let mut base = self.clone();
        let mut e = k;
        while e > 0 {
            if e & 1 == 1 {
                result = result.matmul(&base)?;
            }
            e >>= 1;
            if e > 0 {
                base = base.matmul(&base)?;
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Counting;

    #[test]
    fn power_zero_is_identity() {
        let a = Matrix::from_rows(vec![
            vec![Counting::from_u64(1), Counting::from_u64(1)],
            vec![Counting::from_u64(0), Counting::from_u64(1)],
        ])
        .unwrap();
        assert_eq!(
            a.power(0, AlgebraLimits::default()).unwrap(),
            Matrix::identity(2)
        );
    }

    #[test]
    fn power_counts_walks() {
        // Triangle cycle 0->1->2->0. A^2[i][j] = number of length-2 walks.
        let mut a = Matrix::new(3, 3);
        a.set(0, 1, Counting::from_u64(1)).unwrap();
        a.set(1, 2, Counting::from_u64(1)).unwrap();
        a.set(2, 0, Counting::from_u64(1)).unwrap();
        let a2 = a.power(2, AlgebraLimits::default()).unwrap();
        // 0->1->2 is the only length-2 walk from 0, ending at 2.
        assert_eq!(a2.get(0, 2).unwrap(), &Counting::from_u64(1));
        assert_eq!(a2.get(0, 0).unwrap(), &Counting::from_u64(0));
        // A^3 returns to the start: exactly one closed length-3 walk per node.
        let a3 = a.power(3, AlgebraLimits::default()).unwrap();
        assert_eq!(a3.get(0, 0).unwrap(), &Counting::from_u64(1));
    }

    #[test]
    fn power_non_square_fails() {
        let a: Matrix<Counting> = Matrix::new(2, 3);
        assert!(matches!(
            a.power(2, AlgebraLimits::default()),
            Err(AlgebraError::ShapeMismatch(_))
        ));
    }
}
