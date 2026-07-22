//! Deterministic cookbook builders for discrete spectral recipes.

use crate::{SpectralError, fwht_i64, spectral_energy, walsh_signature};

/// Report produced by the FWHT signal cookbook recipe.
#[derive(Clone, Debug, PartialEq)]
pub struct FwhtSignalDemo {
    /// Input signal in natural order.
    pub signal: Vec<i64>,
    /// FWHT coefficients in natural order.
    pub coefficients: Vec<i64>,
    /// Sum of squared coefficients.
    pub energy: f64,
    /// Low-order Walsh signature.
    pub signature: Vec<f64>,
}

/// Build the modeled FWHT signal report used by the cookbook recipe.
pub fn fwht_signal_demo() -> Result<FwhtSignalDemo, SpectralError> {
    let signal = vec![1, 0, 0, 0];
    let coefficients = fwht_i64(&signal)?.values;
    let coefficients_f64 = coefficients
        .iter()
        .map(|value| *value as f64)
        .collect::<Vec<_>>();
    let energy = spectral_energy(&coefficients_f64);
    let signature = walsh_signature(&coefficients_f64, 4);

    Ok(FwhtSignalDemo {
        signal,
        coefficients,
        energy,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fwht_demo_computes_coefficients_energy_and_signature() {
        let demo = fwht_signal_demo().expect("valid FWHT signal demo");

        assert_eq!(demo.coefficients, vec![1, 1, 1, 1]);
        assert_eq!(demo.energy, 4.0);
        assert_eq!(demo.signature, vec![1.0, 1.0, 1.0, 1.0]);
    }
}
