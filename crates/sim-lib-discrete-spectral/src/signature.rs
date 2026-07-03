//! Walsh-domain signatures: compact features of a transformed signal.
//!
//! These invariants (low-order signature, energy, spectral entropy) are what the
//! rank grade compilers use to grade signals and melodies.

/// The first `order` coefficients (the low-order Walsh signature). If `order`
/// exceeds the length, the whole coefficient vector is returned.
pub fn walsh_signature(coeffs: &[f64], order: usize) -> Vec<f64> {
    coeffs.iter().take(order).copied().collect()
}

/// Total spectral energy: the sum of squared coefficients.
pub fn spectral_energy(coeffs: &[f64]) -> f64 {
    coeffs.iter().map(|c| c * c).sum()
}

/// Shannon entropy (base 2) of the normalized squared-coefficient distribution.
///
/// Returns `0.0` when the signal has no energy. A pure single-coefficient
/// spectrum has entropy `0`; a flat spectrum over `2^k` coefficients has entropy
/// `k`.
pub fn spectral_entropy(coeffs: &[f64]) -> f64 {
    let energy = spectral_energy(coeffs);
    if energy == 0.0 {
        return 0.0;
    }
    let mut h = 0.0;
    for c in coeffs {
        let p = (c * c) / energy;
        if p > 0.0 {
            h -= p * p.log2();
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_takes_low_order() {
        assert_eq!(walsh_signature(&[1.0, 2.0, 3.0, 4.0], 2), vec![1.0, 2.0]);
        assert_eq!(walsh_signature(&[1.0], 5), vec![1.0]);
    }

    #[test]
    fn energy_is_sum_of_squares() {
        assert_eq!(spectral_energy(&[3.0, 4.0]), 25.0);
    }

    #[test]
    fn entropy_extremes() {
        // A single nonzero coefficient -> zero entropy.
        assert_eq!(spectral_entropy(&[5.0, 0.0, 0.0, 0.0]), 0.0);
        // A flat spectrum over 4 coefficients -> entropy 2 bits.
        let flat = spectral_entropy(&[1.0, 1.0, 1.0, 1.0]);
        assert!((flat - 2.0).abs() < 1e-12);
        // Empty / zero-energy -> 0.
        assert_eq!(spectral_entropy(&[0.0, 0.0]), 0.0);
    }
}
