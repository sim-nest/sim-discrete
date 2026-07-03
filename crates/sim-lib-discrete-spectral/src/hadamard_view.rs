//! An implicit Hadamard matrix view, applied via the FWHT.
//!
//! The natural-order Hadamard matrix `H_n` times a vector equals that vector's
//! FWHT, so `matvec` runs in `O(n log n)` without materializing `H_n`. A dense
//! materialization is available behind an explicit limit, reusing the algebra
//! crate's matrix-view contract.

use crate::error::SpectralError;
use crate::transform::{fwht_f64, is_power_of_two_len};
use sim_lib_discrete_algebra::{AlgebraLimits, Matrix, RealF64};

/// An implicit `n x n` natural-order Hadamard matrix, with `n` a power of two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HadamardView {
    /// The dimension `n` (must be a power of two).
    pub n: usize,
}

impl HadamardView {
    /// Apply `H_n` to `v` via the FWHT: `H_n * v == fwht(v)`.
    pub fn matvec(&self, v: &[f64]) -> Result<Vec<f64>, SpectralError> {
        if v.len() != self.n {
            return Err(SpectralError::ShapeMismatch(format!(
                "matvec: vector len {} != view dimension {}",
                v.len(),
                self.n
            )));
        }
        Ok(fwht_f64(v)?.values)
    }

    /// Materialize `H_n` densely (entry `(i, j) = (-1)^popcount(i & j)`), guarded
    /// by `limits`.
    pub fn materialize(&self, limits: AlgebraLimits) -> Result<Matrix<RealF64>, SpectralError> {
        if !is_power_of_two_len(self.n) {
            return Err(SpectralError::NonPowerOfTwoLength { len: self.n });
        }
        if self.n > limits.max_dim {
            return Err(SpectralError::LimitExceeded(format!(
                "materialize: dimension {} exceeds max_dim {}",
                self.n, limits.max_dim
            )));
        }
        let n = self.n;
        let mut m = Matrix::filled(n, n, RealF64(0.0));
        for i in 0..n {
            for j in 0..n {
                let sign = if (i & j).count_ones() % 2 == 0 {
                    1.0
                } else {
                    -1.0
                };
                m.data[i * n + j] = RealF64(sign);
            }
        }
        Ok(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matvec_matches_materialized_matrix() {
        for &n in &[2usize, 4, 8] {
            let v: Vec<f64> = (0..n).map(|i| i as f64 + 1.0).collect();
            let view = HadamardView { n };
            let mv = view.matvec(&v).unwrap();
            let h = view.materialize(AlgebraLimits::default()).unwrap();
            // Dense H * v as a column matmul.
            let col = Matrix::from_rows(v.iter().map(|x| vec![RealF64(*x)]).collect()).unwrap();
            let prod = h.matmul(&col).unwrap();
            for (p, m) in prod.data.iter().zip(&mv) {
                assert!((p.0 - m).abs() < 1e-9, "n={n}");
            }
        }
    }

    #[test]
    fn oversize_materialization_fails_by_limit() {
        let view = HadamardView { n: 128 };
        let limited = AlgebraLimits { max_dim: 16 };
        assert!(matches!(
            view.materialize(limited),
            Err(SpectralError::LimitExceeded(_))
        ));
    }

    #[test]
    fn materialized_is_an_involution_scaled() {
        // H_2 squared = 2 * I.
        let h = HadamardView { n: 2 }
            .materialize(AlgebraLimits::default())
            .unwrap();
        let h2 = h.matmul(&h).unwrap();
        assert_eq!(h2.data[0], RealF64(2.0));
        assert_eq!(h2.data[1], RealF64(0.0));
    }
}
