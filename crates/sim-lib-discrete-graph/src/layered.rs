//! Certified shortest paths through staged state layers.

// conformance: layered dynamic programming verifies stable backpointer
// certificates, forbidden transitions, work bounds, and finite costs.

use core::cmp::Ordering;

use crate::{
    AlgorithmControl, AlgorithmInterrupt, AlgorithmReceipt, FiniteCost, GraphError, NeverInterrupt,
    control::WorkMeter,
    cost::{add, compare, validate},
};

/// One Bellman-table cell in a layered shortest-path certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayeredCell<C> {
    /// Best total cost from the first layer through this state.
    pub total_cost: Option<C>,
    /// Stable index of the predecessor in the previous layer.
    pub predecessor: Option<usize>,
}

/// Bellman values and backpointers proving a layered optimum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayeredCertificate<C> {
    /// One row per input layer, in input state order.
    pub layers: Vec<Vec<LayeredCell<C>>>,
}

/// A minimum-cost path selecting one state from every layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayeredPath<S, C> {
    /// Selected state from each layer.
    pub states: Vec<S>,
    /// Stable input index selected from each layer.
    pub indices: Vec<usize>,
    /// Exact total transition cost.
    pub total_cost: C,
    /// Checkable Bellman values and backpointers.
    pub certificate: LayeredCertificate<C>,
    /// Deterministic cell/edge work accounting.
    pub receipt: AlgorithmReceipt,
}

/// Finds the minimum-cost path selecting one state from every non-empty layer.
///
/// Every transition is allowed. Equal-cost candidates prefer the lower
/// predecessor index, and equal-cost endpoints prefer the lower final index.
pub fn layered_shortest_path<S: Clone, C: FiniteCost>(
    layers: &[Vec<S>],
    cost: impl Fn(&S, &S) -> C,
) -> Result<LayeredPath<S, C>, GraphError> {
    layered_shortest_path_with_control(
        layers,
        |left, right| Some(cost(left, right)),
        &AlgorithmControl::default(),
        &NeverInterrupt,
    )
}

/// Finds a layered shortest path with forbidden transitions and explicit
/// resource control.
///
/// The transition callback returns `None` for a forbidden edge.
pub fn layered_shortest_path_with_control<S: Clone, C: FiniteCost>(
    layers: &[Vec<S>],
    transition: impl Fn(&S, &S) -> Option<C>,
    control: &AlgorithmControl,
    interrupt: &dyn AlgorithmInterrupt,
) -> Result<LayeredPath<S, C>, GraphError> {
    validate_layers(layers)?;
    let memory = layer_cell_count(layers)?;
    let mut meter = WorkMeter::new(control, interrupt, memory)?;
    let certificate = build_certificate(layers, &transition, Some(&mut meter))?;
    let (indices, total_cost) = reconstruct(&certificate)?;
    let states = indices
        .iter()
        .enumerate()
        .map(|(layer, index)| layers[layer][*index].clone())
        .collect();
    Ok(LayeredPath {
        states,
        indices,
        total_cost,
        certificate,
        receipt: meter.finish(),
    })
}

/// Verifies the Bellman recurrence, stable backpointers, selected states, and
/// accounting evidence of a layered shortest path.
pub fn verify_layered_path<S: Clone + PartialEq, C: FiniteCost>(
    layers: &[Vec<S>],
    transition: impl Fn(&S, &S) -> Option<C>,
    path: &LayeredPath<S, C>,
) -> Result<(), GraphError> {
    validate_layers(layers)?;
    path.receipt.validate()?;
    let expected = build_certificate(layers, &transition, None)?;
    if path.certificate != expected {
        return Err(GraphError::CertificateInvalid(
            "layered certificate violates the Bellman recurrence or stable ties".to_owned(),
        ));
    }
    let (indices, total_cost) = reconstruct(&expected)?;
    if path.indices != indices || path.total_cost != total_cost {
        return Err(GraphError::CertificateInvalid(
            "layered path does not select the certified optimum".to_owned(),
        ));
    }
    let states = indices
        .iter()
        .enumerate()
        .map(|(layer, index)| layers[layer][*index].clone())
        .collect::<Vec<_>>();
    if path.states != states {
        return Err(GraphError::CertificateInvalid(
            "layered path states do not match their certified indices".to_owned(),
        ));
    }
    verify_receipt(layers, &path.receipt)
}

fn build_certificate<S, C: FiniteCost>(
    layers: &[Vec<S>],
    transition: &impl Fn(&S, &S) -> Option<C>,
    mut meter: Option<&mut WorkMeter<'_>>,
) -> Result<LayeredCertificate<C>, GraphError> {
    let mut table = Vec::with_capacity(layers.len());
    let mut first = Vec::with_capacity(layers[0].len());
    for _ in &layers[0] {
        charge_cell(&mut meter)?;
        first.push(LayeredCell {
            total_cost: Some(C::zero()),
            predecessor: None,
        });
    }
    table.push(first);

    for layer_index in 1..layers.len() {
        let previous_states = &layers[layer_index - 1];
        let current_states = &layers[layer_index];
        let previous_cells = &table[layer_index - 1];
        let mut current_cells = Vec::with_capacity(current_states.len());
        for current in current_states {
            charge_cell(&mut meter)?;
            let mut best: Option<(C, usize)> = None;
            for (predecessor, previous) in previous_states.iter().enumerate() {
                charge_edge(&mut meter)?;
                let Some(previous_cost) = &previous_cells[predecessor].total_cost else {
                    continue;
                };
                let Some(edge_cost) = transition(previous, current) else {
                    continue;
                };
                validate(&edge_cost, "layered transition")?;
                let candidate = add(previous_cost, &edge_cost, "layered path transition")?;
                let replace = match &best {
                    Some((best_cost, _)) => {
                        compare(&candidate, best_cost, "layered path ordering")? == Ordering::Less
                    }
                    None => true,
                };
                if replace {
                    best = Some((candidate, predecessor));
                }
            }
            current_cells.push(match best {
                Some((total_cost, predecessor)) => LayeredCell {
                    total_cost: Some(total_cost),
                    predecessor: Some(predecessor),
                },
                None => LayeredCell {
                    total_cost: None,
                    predecessor: None,
                },
            });
        }
        table.push(current_cells);
    }
    Ok(LayeredCertificate { layers: table })
}

fn reconstruct<C: FiniteCost>(
    certificate: &LayeredCertificate<C>,
) -> Result<(Vec<usize>, C), GraphError> {
    let last = certificate.layers.last().ok_or_else(|| {
        GraphError::Unsupported("layered path requires at least one layer".to_owned())
    })?;
    let mut endpoint: Option<(usize, &C)> = None;
    for (index, cell) in last.iter().enumerate() {
        let Some(cost) = &cell.total_cost else {
            continue;
        };
        let replace = match endpoint {
            Some((_, best)) => compare(cost, best, "layered endpoint ordering")? == Ordering::Less,
            None => true,
        };
        if replace {
            endpoint = Some((index, cost));
        }
    }
    let (mut index, total_cost) = endpoint.ok_or(GraphError::Disconnected)?;
    let mut indices = vec![0; certificate.layers.len()];
    for layer in (0..certificate.layers.len()).rev() {
        indices[layer] = index;
        if layer > 0 {
            index = certificate.layers[layer][index]
                .predecessor
                .ok_or_else(|| {
                    GraphError::CertificateInvalid(
                        "reachable layered cell has no backpointer".to_owned(),
                    )
                })?;
        }
    }
    Ok((indices, total_cost.clone()))
}

fn validate_layers<S>(layers: &[Vec<S>]) -> Result<(), GraphError> {
    if layers.is_empty() {
        return Err(GraphError::Unsupported(
            "layered path requires at least one layer".to_owned(),
        ));
    }
    if layers.iter().any(Vec::is_empty) {
        return Err(GraphError::Disconnected);
    }
    layer_cell_count(layers).map(|_| ())
}

fn layer_cell_count<S>(layers: &[Vec<S>]) -> Result<usize, GraphError> {
    layers.iter().try_fold(0usize, |total, layer| {
        total
            .checked_add(layer.len())
            .ok_or_else(|| GraphError::WeightOverflow("layer cell count".to_owned()))
    })
}

fn verify_receipt<S>(layers: &[Vec<S>], receipt: &AlgorithmReceipt) -> Result<(), GraphError> {
    let cells = u64::try_from(layer_cell_count(layers)?)
        .map_err(|_| GraphError::CertificateInvalid("layer cell count exceeds u64".to_owned()))?;
    let edges = layers.windows(2).try_fold(0u64, |total, pair| {
        let count = pair[0]
            .len()
            .checked_mul(pair[1].len())
            .and_then(|count| u64::try_from(count).ok())
            .ok_or_else(|| {
                GraphError::CertificateInvalid("layer edge count exceeds u64".to_owned())
            })?;
        total
            .checked_add(count)
            .ok_or_else(|| GraphError::CertificateInvalid("layer edge count overflowed".to_owned()))
    })?;
    if receipt.cells != cells
        || receipt.edges != edges
        || receipt.peak_memory_cells != layer_cell_count(layers)?
    {
        return Err(GraphError::CertificateInvalid(
            "layered path receipt counts do not match the input".to_owned(),
        ));
    }
    Ok(())
}

fn charge_cell(meter: &mut Option<&mut WorkMeter<'_>>) -> Result<(), GraphError> {
    if let Some(meter) = meter.as_deref_mut() {
        meter.cell()?;
    }
    Ok(())
}

fn charge_edge(meter: &mut Option<&mut WorkMeter<'_>>) -> Result<(), GraphError> {
    if let Some(meter) = meter.as_deref_mut() {
        meter.edge()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_path_has_stable_backpointer_certificate() {
        let layers = vec![vec!['a', 'b'], vec!['c', 'd'], vec!['e']];
        let path = layered_shortest_path(&layers, |left, right| {
            if (*left, *right) == ('a', 'd') || (*left, *right) == ('d', 'e') {
                1_i64
            } else {
                5
            }
        })
        .expect("layered path");

        assert_eq!(path.indices, vec![0, 1, 0]);
        assert_eq!(path.states, vec!['a', 'd', 'e']);
        assert_eq!(path.total_cost, 2);
        assert_eq!(path.receipt.cells, 5);
        assert_eq!(path.receipt.edges, 6);
        verify_layered_path(
            &layers,
            |left, right| {
                Some(
                    if (*left, *right) == ('a', 'd') || (*left, *right) == ('d', 'e') {
                        1_i64
                    } else {
                        5
                    },
                )
            },
            &path,
        )
        .expect("certificate");
    }

    #[test]
    fn forbidden_edges_bounds_and_non_finite_costs_fail_closed() {
        let layers = vec![vec![0, 1], vec![2], vec![3]];
        let path = layered_shortest_path_with_control(
            &layers,
            |left, right| (*left != 0 || *right != 2).then_some(1_i64),
            &AlgorithmControl::default(),
            &NeverInterrupt,
        )
        .expect("reachable through state one");
        assert_eq!(path.indices, vec![1, 0, 0]);

        assert!(matches!(
            layered_shortest_path_with_control(
                &layers,
                |_left, _right| Some(1_i64),
                &AlgorithmControl::default().with_max_work(2),
                &NeverInterrupt,
            ),
            Err(GraphError::ControlStopped(_))
        ));
        assert!(matches!(
            layered_shortest_path(&layers, |_left, _right| f64::INFINITY),
            Err(GraphError::NonFiniteCost(_))
        ));
    }

    #[test]
    fn tampered_backpointer_fails_verification() {
        let layers = vec![vec![0, 1], vec![2, 3]];
        let mut path = layered_shortest_path(&layers, |_left, _right| 1_i64).expect("layered path");
        path.certificate.layers[1][0].predecessor = Some(1);
        assert!(matches!(
            verify_layered_path(&layers, |_left, _right| Some(1_i64), &path),
            Err(GraphError::CertificateInvalid(_))
        ));
    }
}
