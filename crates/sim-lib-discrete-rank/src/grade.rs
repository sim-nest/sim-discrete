//! Grade compilers: synthesize a rank grade (lower = simpler) from an algebraic
//! or spectral invariant, by calling the spine and the spectral crate.

use crate::error::RankAdapterError;
use sim_lib_discrete_algebra::BoolRing;
use sim_lib_discrete_graph::{Graph, kruskals_mst, reachability};
use sim_lib_discrete_spectral::{fwht_f64, spectral_entropy};

/// `grade/discrete/mst-weight`: grade an undirected weighted graph by its MST
/// total weight. Lower weight = simpler. Errors if the graph is disconnected.
pub fn mst_weight_grade<N>(graph: &Graph<N, u64>) -> Result<u64, RankAdapterError> {
    Ok(kruskals_mst(graph)?.total_weight)
}

/// `grade/discrete/closure-rank`: grade a graph by the density of its transitive
/// closure (number of reachable ordered pairs). Denser = higher grade.
pub fn closure_density_grade<N, W>(graph: &Graph<N, W>) -> Result<u64, RankAdapterError> {
    let r = reachability(graph)?;
    let count = r.data.iter().filter(|c| **c == BoolRing(true)).count();
    Ok(count as u64)
}

/// `grade/discrete/spectral-entropy`: the Walsh spectral entropy of a signal
/// (its FWHT is taken internally). Lower entropy = simpler = lower grade.
pub fn signal_spectral_entropy_grade(signal: &[f64]) -> Result<f64, RankAdapterError> {
    let coeffs = fwht_f64(signal)?;
    Ok(spectral_entropy(&coeffs.values))
}

/// Quantize a spectral entropy into `bands` ordinal grade bands `0..bands`,
/// normalized by the maximum entropy `log2(len)`.
pub fn spectral_entropy_band(signal: &[f64], bands: u64) -> Result<u64, RankAdapterError> {
    if bands == 0 {
        return Err(RankAdapterError::Invalid("bands must be > 0".to_string()));
    }
    let entropy = signal_spectral_entropy_grade(signal)?;
    let max = (signal.len().max(2) as f64).log2();
    if max <= 0.0 {
        return Ok(0);
    }
    let frac = (entropy / max).clamp(0.0, 1.0);
    let band = (frac * bands as f64).floor() as u64;
    Ok(band.min(bands - 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_lib_discrete_graph::Directedness;

    #[test]
    fn mst_weight_grade_is_monotone() {
        let light = {
            let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Undirected);
            g.add_edge(0, 1, 1).unwrap();
            g.add_edge(1, 2, 1).unwrap();
            g
        };
        let heavy = {
            let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Undirected);
            g.add_edge(0, 1, 5).unwrap();
            g.add_edge(1, 2, 5).unwrap();
            g
        };
        assert!(mst_weight_grade(&light).unwrap() < mst_weight_grade(&heavy).unwrap());
    }

    #[test]
    fn closure_density_grade_is_monotone() {
        // A directed chain 0->1->2 reaches fewer pairs than adding 2->0 (cycle).
        let chain = {
            let mut g: Graph<u8, u64> = Graph::with_nodes(vec![0, 1, 2], Directedness::Directed);
            g.add_edge(0, 1, 1).unwrap();
            g.add_edge(1, 2, 1).unwrap();
            g
        };
        let cycle = {
            let mut g = chain.clone();
            g.add_edge(2, 0, 1).unwrap();
            g
        };
        assert!(closure_density_grade(&chain).unwrap() < closure_density_grade(&cycle).unwrap());
    }

    #[test]
    fn spectral_entropy_grade_is_monotone() {
        // Constant signal -> energy concentrated in one Walsh coefficient -> low
        // entropy. An impulse -> flat spectrum -> high entropy.
        let constant = signal_spectral_entropy_grade(&[1.0, 1.0, 1.0, 1.0]).unwrap();
        let impulse = signal_spectral_entropy_grade(&[1.0, 0.0, 0.0, 0.0]).unwrap();
        assert!(constant < impulse, "{constant} vs {impulse}");
        assert_eq!(spectral_entropy_band(&[1.0, 1.0, 1.0, 1.0], 4).unwrap(), 0);
        assert_eq!(spectral_entropy_band(&[1.0, 0.0, 0.0, 0.0], 4).unwrap(), 3);
    }
}
