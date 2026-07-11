#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Discrete spectral atlas.
//!
//! This crate hosts the Fast Walsh-Hadamard Transform framed as the Fourier /
//! character transform of the boolean hypercube group `(Z/2)^n`: FWHT / IFWHT,
//! XOR convolution, subset zeta / Mobius transforms, Walsh signatures, and an
//! implicit Hadamard matrix view. Music is a downstream consumer; no music type
//! ever appears here.
//!
//! Boundary: depends on `sim-lib-discrete-algebra`; never on `sim-lib-music-*`.

pub mod convolution;
pub mod error;
pub mod hadamard_view;
pub mod signal;
pub mod signature;
pub mod subset;
pub mod transform;

pub use convolution::{xor_convolution_f64, xor_convolution_i64};
pub use error::SpectralError;
pub use hadamard_view::HadamardView;
pub use signal::{Normalization, WalshBasis, WalshSignal};
pub use signature::{spectral_energy, spectral_entropy, walsh_signature};
pub use subset::{mobius_transform, or_convolution, subset_convolution, zeta_transform};
pub use transform::{
    fwht_f64, fwht_f64_in_place, fwht_i64, fwht_i64_in_place, ifwht_f64, ifwht_f64_in_place,
    ifwht_i64, ifwht_i64_in_place, is_power_of_two_len, next_power_of_two_len, pad_to_power_of_two,
};

/// Cookbook recipes for this lib, embedded at build time.
pub static RECIPES: sim_cookbook::EmbeddedDir =
    include!(concat!(env!("OUT_DIR"), "/cookbook_recipes.rs"));
