//! XOR convolution via the FWHT: `c[k] = sum_i a[i] * b[i ^ k]`.
//!
//! The transform diagonalizes XOR convolution, so `c = ifwht(fwht(a) .* fwht(b))`.

use crate::error::SpectralError;
use crate::signal::Normalization;
use crate::transform::{fwht_f64, fwht_i64, ifwht_f64, ifwht_i64};

fn same_len<T>(a: &[T], b: &[T]) -> Result<(), SpectralError> {
    if a.len() != b.len() {
        return Err(SpectralError::ShapeMismatch(format!(
            "convolution operands differ: {} vs {}",
            a.len(),
            b.len()
        )));
    }
    Ok(())
}

/// XOR convolution over `i64`. Input length must be a power of two.
///
/// `c[k] = sum over (i ^ j == k) of a[i] * b[j]`, computed in the Walsh domain.
///
/// # Examples
///
/// Convolving against the delta `[1, 0, 0, 0]` (the XOR identity) returns the
/// other input unchanged:
///
/// ```
/// use sim_lib_discrete_spectral::xor_convolution_i64;
///
/// let a = [1, 2, 3, 4];
/// let delta = [1, 0, 0, 0];
/// assert_eq!(xor_convolution_i64(&a, &delta).unwrap(), vec![1, 2, 3, 4]);
/// ```
pub fn xor_convolution_i64(a: &[i64], b: &[i64]) -> Result<Vec<i64>, SpectralError> {
    same_len(a, b)?;
    let fa = fwht_i64(a)?;
    let fb = fwht_i64(b)?;
    let prod: Vec<i64> = fa
        .values
        .iter()
        .zip(&fb.values)
        .map(|(x, y)| x.checked_mul(*y).ok_or(SpectralError::Overflow))
        .collect::<Result<_, _>>()?;
    ifwht_i64(&prod, Normalization::DivideByLength)
}

/// XOR convolution over `f64`. Input length must be a power of two.
pub fn xor_convolution_f64(a: &[f64], b: &[f64]) -> Result<Vec<f64>, SpectralError> {
    same_len(a, b)?;
    let fa = fwht_f64(a)?;
    let fb = fwht_f64(b)?;
    let prod: Vec<f64> = fa
        .values
        .iter()
        .zip(&fb.values)
        .map(|(x, y)| x * y)
        .collect();
    ifwht_f64(&prod, Normalization::DivideByLength)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn naive_xor(a: &[i64], b: &[i64]) -> Vec<i64> {
        let n = a.len();
        (0..n)
            .map(|k| (0..n).map(|i| a[i] * b[i ^ k]).sum())
            .collect()
    }

    #[test]
    fn matches_naive_len4() {
        let a = [1, 2, 3, 4];
        let b = [5, 6, 7, 8];
        assert_eq!(xor_convolution_i64(&a, &b).unwrap(), naive_xor(&a, &b));
    }

    #[test]
    fn matches_naive_len16() {
        let a: Vec<i64> = (0..16).map(|i| i * 2 - 7).collect();
        let b: Vec<i64> = (0..16).map(|i| (i * i) % 5 - 2).collect();
        assert_eq!(xor_convolution_i64(&a, &b).unwrap(), naive_xor(&a, &b));
    }

    #[test]
    fn f64_matches_integer_case() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [5.0, 6.0, 7.0, 8.0];
        let got = xor_convolution_f64(&a, &b).unwrap();
        let want = naive_xor(&[1, 2, 3, 4], &[5, 6, 7, 8]);
        for (x, y) in got.iter().zip(&want) {
            assert!((x - *y as f64).abs() < 1e-9);
        }
    }

    #[test]
    fn non_power_of_two_fails() {
        assert!(xor_convolution_i64(&[1, 2, 3], &[1, 2, 3]).is_err());
    }

    #[test]
    fn moderate_coefficients_overflow_reports_err() {
        // Legal ~3e9 coefficients: the FWHT spectra reach ~1.2e10 each, whose
        // pointwise product overflows i64. Must be Err, not a silent wrong answer.
        let a = vec![3_000_000_000i64; 4];
        let b = vec![3_000_000_000i64; 4];
        assert_eq!(
            xor_convolution_i64(&a, &b).unwrap_err(),
            SpectralError::Overflow
        );
    }
}
