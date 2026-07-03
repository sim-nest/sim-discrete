//! Subset-sum (zeta) and Mobius transforms over the subset lattice, and the
//! disjoint subset-sum convolution built from them.
//!
//! These are the same hypercube butterfly as the FWHT, but with `add`-only
//! (zeta) and `subtract`-only (Mobius) merges instead of the `+/-` pair.

use crate::error::SpectralError;
use crate::transform::is_power_of_two_len;

fn require_pow2(a: &[f64]) -> Result<usize, SpectralError> {
    if !is_power_of_two_len(a.len()) {
        return Err(SpectralError::NonPowerOfTwoLength { len: a.len() });
    }
    Ok(a.len().trailing_zeros() as usize)
}

/// Sum over subsets (zeta): `out[mask] = sum_{s subseteq mask} a[s]`.
pub fn zeta_transform(a: &[f64]) -> Result<Vec<f64>, SpectralError> {
    let bits = require_pow2(a)?;
    let mut f = a.to_vec();
    let n = f.len();
    for i in 0..bits {
        for mask in 0..n {
            if mask & (1 << i) != 0 {
                f[mask] += f[mask ^ (1 << i)];
            }
        }
    }
    Ok(f)
}

/// The inverse of [`zeta_transform`] (Mobius): recovers `a` from its subset sums.
pub fn mobius_transform(a: &[f64]) -> Result<Vec<f64>, SpectralError> {
    let bits = require_pow2(a)?;
    let mut f = a.to_vec();
    let n = f.len();
    for i in 0..bits {
        for mask in 0..n {
            if mask & (1 << i) != 0 {
                f[mask] -= f[mask ^ (1 << i)];
            }
        }
    }
    Ok(f)
}

/// OR convolution: `c[k] = sum_{i | j = k} a[i] * b[j]`, via zeta / Mobius.
pub fn or_convolution(a: &[f64], b: &[f64]) -> Result<Vec<f64>, SpectralError> {
    if a.len() != b.len() {
        return Err(SpectralError::ShapeMismatch(
            "or_convolution operands".to_string(),
        ));
    }
    let za = zeta_transform(a)?;
    let zb = zeta_transform(b)?;
    let prod: Vec<f64> = za.iter().zip(&zb).map(|(x, y)| x * y).collect();
    mobius_transform(&prod)
}

// Internal zeta over a single popcount layer (no error checks; caller ensures
// the length is a power of two).
fn zeta_in_place(f: &mut [f64], bits: usize) {
    let n = f.len();
    for i in 0..bits {
        for mask in 0..n {
            if mask & (1 << i) != 0 {
                f[mask] += f[mask ^ (1 << i)];
            }
        }
    }
}

fn mobius_in_place(f: &mut [f64], bits: usize) {
    let n = f.len();
    for i in 0..bits {
        for mask in 0..n {
            if mask & (1 << i) != 0 {
                f[mask] -= f[mask ^ (1 << i)];
            }
        }
    }
}

/// Disjoint subset-sum convolution: `h[s] = sum_{t subseteq s} a[t] * b[s\t]`,
/// via the popcount-ranked Mobius method.
pub fn subset_convolution(a: &[f64], b: &[f64]) -> Result<Vec<f64>, SpectralError> {
    if a.len() != b.len() {
        return Err(SpectralError::ShapeMismatch(
            "subset_convolution operands".to_string(),
        ));
    }
    let bits = require_pow2(a)?;
    let n = a.len();
    let mut fa = vec![vec![0.0; n]; bits + 1];
    let mut fb = vec![vec![0.0; n]; bits + 1];
    for (mask, (&av, &bv)) in a.iter().zip(b).enumerate() {
        let pc = (mask as u32).count_ones() as usize;
        fa[pc][mask] = av;
        fb[pc][mask] = bv;
    }
    for layer in 0..=bits {
        zeta_in_place(&mut fa[layer], bits);
        zeta_in_place(&mut fb[layer], bits);
    }
    let mut fh = vec![vec![0.0; n]; bits + 1];
    for i in 0..=bits {
        for j in 0..=i {
            for mask in 0..n {
                fh[i][mask] += fa[j][mask] * fb[i - j][mask];
            }
        }
    }
    let mut h = vec![0.0; n];
    for (i, layer) in fh.iter_mut().enumerate() {
        mobius_in_place(layer, bits);
        for mask in 0..n {
            if (mask as u32).count_ones() as usize == i {
                h[mask] = layer[mask];
            }
        }
    }
    Ok(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeta_mobius_are_inverse() {
        let a = vec![3.0, -1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
        let recovered = mobius_transform(&zeta_transform(&a).unwrap()).unwrap();
        for (x, y) in recovered.iter().zip(&a) {
            assert!((x - y).abs() < 1e-9);
        }
    }

    #[test]
    fn zeta_is_subset_sum() {
        // n=4: out[0b11] = a[00]+a[01]+a[10]+a[11].
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let z = zeta_transform(&a).unwrap();
        assert_eq!(z[3], 10.0);
        assert_eq!(z[1], 1.0 + 2.0);
    }

    #[test]
    fn subset_convolution_matches_brute_force() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let got = subset_convolution(&a, &b).unwrap();
        // Brute force: h[s] = sum over t subset s of a[t]*b[s ^ t].
        let n = a.len();
        let mut want = vec![0.0; n];
        for s in 0..n {
            let mut sub = s;
            loop {
                want[s] += a[sub] * b[s ^ sub];
                if sub == 0 {
                    break;
                }
                sub = (sub - 1) & s;
            }
        }
        for (x, y) in got.iter().zip(&want) {
            assert!((x - y).abs() < 1e-9, "{x} vs {y}");
        }
    }
}
