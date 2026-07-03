//! The Fast Walsh-Hadamard Transform: the character transform of `(Z/2)^n`.
//!
//! The in-place routines apply the natural-order Hadamard butterfly. The
//! transform is its own inverse up to the scale factor `n`, so an inverse
//! divides by the length (or leaves scaling to the caller).

use crate::error::SpectralError;
use crate::signal::{Normalization, WalshSignal};

/// Whether `len` is a power of two (and non-zero).
pub fn is_power_of_two_len(len: usize) -> bool {
    len != 0 && len & (len - 1) == 0
}

/// The smallest power of two `>= len` (with `0 -> 1`).
pub fn next_power_of_two_len(len: usize) -> Result<usize, SpectralError> {
    if len <= 1 {
        return Ok(1);
    }
    let mut p: usize = 1;
    while p < len {
        p = p
            .checked_mul(2)
            .ok_or(SpectralError::LengthOverflow { len })?;
    }
    Ok(p)
}

/// Pad `values` with `fill` up to the next power of two; returns the padded
/// vector and its length.
pub fn pad_to_power_of_two<T: Clone>(
    values: &[T],
    fill: T,
) -> Result<(Vec<T>, usize), SpectralError> {
    let target = next_power_of_two_len(values.len())?;
    let mut out = values.to_vec();
    out.resize(target, fill);
    Ok((out, target))
}

macro_rules! butterfly {
    ($a:expr) => {{
        let n = $a.len();
        let mut h = 1;
        while h < n {
            let mut i = 0;
            while i < n {
                for j in i..i + h {
                    let x = $a[j];
                    let y = $a[j + h];
                    $a[j] = x + y;
                    $a[j + h] = x - y;
                }
                i += h * 2;
            }
            h *= 2;
        }
    }};
}

/// The Hadamard butterfly over `i64` with checked arithmetic.
///
/// Returns [`SpectralError::Overflow`] rather than panicking (debug) or silently
/// wrapping (release) when a coefficient sum or difference leaves `i64`. The
/// transform is documented exact, so an overflow is a hard error, not a result.
fn butterfly_i64_checked(a: &mut [i64]) -> Result<(), SpectralError> {
    let n = a.len();
    let mut h = 1;
    while h < n {
        let mut i = 0;
        while i < n {
            for j in i..i + h {
                let x = a[j];
                let y = a[j + h];
                a[j] = x.checked_add(y).ok_or(SpectralError::Overflow)?;
                a[j + h] = x.checked_sub(y).ok_or(SpectralError::Overflow)?;
            }
            i += h * 2;
        }
        h *= 2;
    }
    Ok(())
}

/// In-place forward FWHT over `i64` (natural order, no normalization).
pub fn fwht_i64_in_place(a: &mut [i64]) -> Result<(), SpectralError> {
    if !is_power_of_two_len(a.len()) {
        return Err(SpectralError::NonPowerOfTwoLength { len: a.len() });
    }
    butterfly_i64_checked(a)
}

/// In-place inverse FWHT over `i64` with the given normalization.
///
/// `DivideByLength` rejects non-divisible coefficients with
/// [`SpectralError::NonDivisibleInverse`]. `OrthonormalF64` is invalid here.
pub fn ifwht_i64_in_place(a: &mut [i64], norm: Normalization) -> Result<(), SpectralError> {
    if !is_power_of_two_len(a.len()) {
        return Err(SpectralError::NonPowerOfTwoLength { len: a.len() });
    }
    let n = a.len() as i64;
    butterfly_i64_checked(a)?;
    match norm {
        Normalization::None => Ok(()),
        Normalization::DivideByLength => {
            for v in a.iter() {
                if v % n != 0 {
                    return Err(SpectralError::NonDivisibleInverse { len: a.len() });
                }
            }
            for v in a.iter_mut() {
                *v /= n;
            }
            Ok(())
        }
        Normalization::OrthonormalF64 => Err(SpectralError::InvalidNormalization(
            "OrthonormalF64 is not valid for i64".to_string(),
        )),
    }
}

/// In-place forward FWHT over `f64` (natural order, no normalization).
pub fn fwht_f64_in_place(a: &mut [f64]) -> Result<(), SpectralError> {
    if !is_power_of_two_len(a.len()) {
        return Err(SpectralError::NonPowerOfTwoLength { len: a.len() });
    }
    butterfly!(a);
    Ok(())
}

/// In-place inverse FWHT over `f64` with the given normalization.
pub fn ifwht_f64_in_place(a: &mut [f64], norm: Normalization) -> Result<(), SpectralError> {
    if !is_power_of_two_len(a.len()) {
        return Err(SpectralError::NonPowerOfTwoLength { len: a.len() });
    }
    let n = a.len() as f64;
    butterfly!(a);
    match norm {
        Normalization::None => {}
        Normalization::DivideByLength => {
            for v in a.iter_mut() {
                *v /= n;
            }
        }
        Normalization::OrthonormalF64 => {
            let s = n.sqrt();
            for v in a.iter_mut() {
                *v /= s;
            }
        }
    }
    Ok(())
}

/// Out-of-place forward FWHT over `i64`, returning a natural-basis signal.
///
/// # Examples
///
/// The transform expects a power-of-two length and returns the unnormalized
/// Walsh coefficients:
///
/// ```
/// use sim_lib_discrete_spectral::fwht_i64;
///
/// let sig = fwht_i64(&[1, 2, 3, 4]).unwrap();
/// assert_eq!(sig.values, vec![10, -2, -4, 0]);
///
/// // A non-power-of-two length is rejected.
/// assert!(fwht_i64(&[1, 2, 3]).is_err());
/// ```
pub fn fwht_i64(values: &[i64]) -> Result<WalshSignal<i64>, SpectralError> {
    let mut a = values.to_vec();
    fwht_i64_in_place(&mut a)?;
    Ok(WalshSignal::natural(a))
}

/// Out-of-place inverse FWHT over `i64`.
///
/// # Examples
///
/// With [`Normalization::DivideByLength`], the inverse transform recovers the
/// original signal from its forward FWHT coefficients:
///
/// ```
/// use sim_lib_discrete_spectral::{fwht_i64, ifwht_i64, Normalization};
///
/// let original = vec![1, 2, 3, 4];
/// let spectrum = fwht_i64(&original).unwrap();
/// let recovered = ifwht_i64(&spectrum.values, Normalization::DivideByLength).unwrap();
/// assert_eq!(recovered, original);
/// ```
pub fn ifwht_i64(values: &[i64], norm: Normalization) -> Result<Vec<i64>, SpectralError> {
    let mut a = values.to_vec();
    ifwht_i64_in_place(&mut a, norm)?;
    Ok(a)
}

/// Out-of-place forward FWHT over `f64`, returning a natural-basis signal.
pub fn fwht_f64(values: &[f64]) -> Result<WalshSignal<f64>, SpectralError> {
    let mut a = values.to_vec();
    fwht_f64_in_place(&mut a)?;
    Ok(WalshSignal::natural(a))
}

/// Out-of-place inverse FWHT over `f64`.
pub fn ifwht_f64(values: &[f64], norm: Normalization) -> Result<Vec<f64>, SpectralError> {
    let mut a = values.to_vec();
    ifwht_f64_in_place(&mut a, norm)?;
    Ok(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_of_two_checks() {
        assert!(is_power_of_two_len(1));
        assert!(is_power_of_two_len(8));
        assert!(!is_power_of_two_len(0));
        assert!(!is_power_of_two_len(3));
        assert_eq!(next_power_of_two_len(3).unwrap(), 4);
        assert_eq!(next_power_of_two_len(0).unwrap(), 1);
    }

    #[test]
    fn padding_extends_to_power_of_two() {
        let (padded, len) = pad_to_power_of_two(&[1i64, 2, 3], 0).unwrap();
        assert_eq!(len, 4);
        assert_eq!(padded, vec![1, 2, 3, 0]);
    }

    #[test]
    fn non_power_of_two_input_fails() {
        let mut a = vec![1i64, 2, 3];
        assert!(matches!(
            fwht_i64_in_place(&mut a),
            Err(SpectralError::NonPowerOfTwoLength { len: 3 })
        ));
    }

    #[test]
    fn unit_vector_transforms_to_all_ones() {
        let sig = fwht_i64(&[1, 0, 0, 0]).unwrap();
        assert_eq!(sig.values, vec![1, 1, 1, 1]);
    }

    #[test]
    fn i64_round_trip_is_exact() {
        let original = vec![3i64, 1, 4, 1, 5, 9, 2, 6];
        let fwd = fwht_i64(&original).unwrap();
        let back = ifwht_i64(&fwd.values, Normalization::DivideByLength).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn self_duality_scales_by_length() {
        let original = vec![1i64, 2, 3, 4];
        let once = fwht_i64(&original).unwrap();
        let twice = fwht_i64(&once.values).unwrap();
        let expected: Vec<i64> = original.iter().map(|x| x * 4).collect();
        assert_eq!(twice.values, expected);
    }

    #[test]
    fn f64_round_trip_within_tolerance() {
        let original = vec![0.5f64, -1.25, 2.0, 3.5];
        let fwd = fwht_f64(&original).unwrap();
        let back = ifwht_f64(&fwd.values, Normalization::DivideByLength).unwrap();
        for (a, b) in back.iter().zip(&original) {
            assert!((a - b).abs() < 1e-12, "{a} vs {b}");
        }
    }

    #[test]
    fn max_coefficient_fwht_reports_overflow() {
        // The first butterfly stage sums adjacent coefficients; two near-i64::MAX
        // values overflow. An exact transform must report Err, never a wrong answer.
        let mut a = vec![i64::MAX, i64::MAX];
        assert_eq!(fwht_i64_in_place(&mut a), Err(SpectralError::Overflow));

        // Out-of-place form surfaces the same error.
        assert_eq!(
            fwht_i64(&[i64::MAX, 1]).unwrap_err(),
            SpectralError::Overflow
        );
    }

    #[test]
    fn non_divisible_integer_inverse_rejected() {
        // [1,0,0,0] inverse-divided by 4 is not integer-divisible.
        let err = ifwht_i64(&[1, 0, 0, 0], Normalization::DivideByLength).unwrap_err();
        assert!(matches!(err, SpectralError::NonDivisibleInverse { len: 4 }));
    }
}
