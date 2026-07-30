use super::{
    Assignment, AssignmentCertificate, AssignmentCost, AssignmentOperation, AssignmentPolicy,
    CostMatrix, add, certificate_error, sub,
};
use crate::GraphError;

#[derive(Copy, Clone, Debug)]
struct ArcRef {
    from: usize,
    index: usize,
}

#[derive(Clone, Debug)]
struct Arc<C> {
    to: usize,
    reverse: usize,
    capacity: usize,
    initial_capacity: usize,
    cost: C,
}

#[derive(Clone, Debug)]
struct Network<C> {
    adjacency: Vec<Vec<Arc<C>>>,
}

impl<C: AssignmentCost> Network<C> {
    fn new(nodes: usize) -> Self {
        Self {
            adjacency: vec![Vec::new(); nodes],
        }
    }

    fn add_arc(&mut self, from: usize, to: usize, capacity: usize, cost: C) -> ArcRef {
        let forward_index = self.adjacency[from].len();
        let reverse_index = self.adjacency[to].len();
        let reverse_cost = C::zero()
            .checked_sub(&cost)
            .expect("validated non-negative costs and derived residual costs are negatable");
        self.adjacency[from].push(Arc {
            to,
            reverse: reverse_index,
            capacity,
            initial_capacity: capacity,
            cost,
        });
        self.adjacency[to].push(Arc {
            to: from,
            reverse: forward_index,
            capacity: 0,
            initial_capacity: 0,
            cost: reverse_cost,
        });
        ArcRef {
            from,
            index: forward_index,
        }
    }

    fn flow(&self, arc: ArcRef) -> usize {
        let edge = &self.adjacency[arc.from][arc.index];
        edge.initial_capacity - edge.capacity
    }

    fn send(&mut self, arc: ArcRef, amount: usize) -> Result<(), GraphError> {
        let (to, reverse, capacity) = {
            let edge = &self.adjacency[arc.from][arc.index];
            (edge.to, edge.reverse, edge.capacity)
        };
        if capacity < amount {
            return certificate_error("assignment operation exceeds network capacity");
        }
        self.adjacency[arc.from][arc.index].capacity -= amount;
        self.adjacency[to][reverse].capacity = self.adjacency[to][reverse]
            .capacity
            .checked_add(amount)
            .ok_or_else(|| GraphError::WeightOverflow("residual capacity".to_owned()))?;
        Ok(())
    }

    fn augment_path(
        &mut self,
        source: usize,
        sink: usize,
        predecessors: &[Option<ArcRef>],
    ) -> Result<(), GraphError> {
        let mut node = sink;
        while node != source {
            let arc = predecessors[node].ok_or_else(|| {
                GraphError::InvalidAssignment(
                    "assignment network has no augmenting path".to_owned(),
                )
            })?;
            self.send(arc, 1)?;
            node = arc.from;
        }
        Ok(())
    }
}

struct Layout<C> {
    network: Network<C>,
    source: usize,
    sink: usize,
    source_first: Vec<ArcRef>,
    source_double: Vec<Option<ArcRef>>,
    insertion: Vec<ArcRef>,
    pairs: Vec<Vec<ArcRef>>,
    target_sink: Vec<ArcRef>,
    deletion_base: C,
}

type ResidualPath<C> = (Vec<Option<C>>, Vec<Option<ArcRef>>);

fn build<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
) -> Result<Layout<C>, GraphError> {
    let source = 0;
    let row_base = 1;
    let target_base = row_base + costs.rows();
    let sink = target_base + costs.columns();
    let mut network = Network::new(sink + 1);
    let mut source_first = Vec::with_capacity(costs.rows());
    let mut source_double = Vec::with_capacity(costs.rows());
    let mut deletion_base = C::zero();

    for row in 0..costs.rows() {
        let deletion = &policy.deletion_costs[row];
        deletion_base = add(&deletion_base, deletion, "assignment deletion base")?;
        let credit = sub(&C::zero(), deletion, "assignment deletion credit")?;
        source_first.push(network.add_arc(source, row_base + row, 1, credit));
        source_double.push(policy.doubling_cost(row).map(|doubling| {
            network.add_arc(
                source,
                row_base + row,
                costs.columns().saturating_sub(1),
                doubling.clone(),
            )
        }));
    }

    let insertion = (0..costs.columns())
        .map(|target| {
            network.add_arc(
                source,
                target_base + target,
                1,
                policy.insertion_costs[target].clone(),
            )
        })
        .collect::<Vec<_>>();

    let mut pairs = Vec::with_capacity(costs.rows());
    for row in 0..costs.rows() {
        pairs.push(
            (0..costs.columns())
                .map(|target| {
                    network.add_arc(
                        row_base + row,
                        target_base + target,
                        1,
                        costs.value(row, target).clone(),
                    )
                })
                .collect(),
        );
    }
    let target_sink = (0..costs.columns())
        .map(|target| network.add_arc(target_base + target, sink, 1, C::zero()))
        .collect();

    Ok(Layout {
        network,
        source,
        sink,
        source_first,
        source_double,
        insertion,
        pairs,
        target_sink,
        deletion_base,
    })
}

pub(super) fn solve<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
) -> Result<Assignment<C>, GraphError> {
    let mut layout = build(costs, policy)?;
    let mut flow_cost = C::zero();
    for _ in 0..costs.columns() {
        let (distances, predecessors) = shortest_residual_path(&layout.network, layout.source)?;
        let distance = distances[layout.sink].as_ref().ok_or_else(|| {
            GraphError::InvalidAssignment("assignment network cannot cover every target".to_owned())
        })?;
        flow_cost = add(&flow_cost, distance, "assignment flow total")?;
        layout
            .network
            .augment_path(layout.source, layout.sink, &predecessors)?;
    }

    let operations = operations_from_flow(costs, policy, &layout)?;
    let total_cost = add(
        &layout.deletion_base,
        &flow_cost,
        "assignment objective total",
    )?;
    let potentials = residual_potentials(&layout.network)?;
    Ok(Assignment {
        operations,
        total_cost,
        certificate: AssignmentCertificate::MinCostFlow { potentials },
    })
}

fn shortest_residual_path<C: AssignmentCost>(
    network: &Network<C>,
    source: usize,
) -> Result<ResidualPath<C>, GraphError> {
    let nodes = network.adjacency.len();
    let mut distances = vec![None; nodes];
    let mut predecessors = vec![None; nodes];
    distances[source] = Some(C::zero());
    for _ in 0..nodes.saturating_sub(1) {
        let mut changed = false;
        for from in 0..nodes {
            let Some(distance) = distances[from].clone() else {
                continue;
            };
            for (index, edge) in network.adjacency[from].iter().enumerate() {
                if edge.capacity == 0 {
                    continue;
                }
                let candidate = add(&distance, &edge.cost, "assignment path relaxation")?;
                if distances[edge.to]
                    .as_ref()
                    .is_none_or(|current| candidate < *current)
                {
                    distances[edge.to] = Some(candidate);
                    predecessors[edge.to] = Some(ArcRef { from, index });
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    Ok((distances, predecessors))
}

fn residual_potentials<C: AssignmentCost>(network: &Network<C>) -> Result<Vec<C>, GraphError> {
    let nodes = network.adjacency.len();
    let mut potentials = vec![C::zero(); nodes];
    for iteration in 0..nodes {
        let mut changed = false;
        for from in 0..nodes {
            for edge in &network.adjacency[from] {
                if edge.capacity == 0 {
                    continue;
                }
                let candidate = add(&potentials[from], &edge.cost, "assignment dual relaxation")?;
                if candidate < potentials[edge.to] {
                    potentials[edge.to] = candidate;
                    changed = true;
                    if iteration + 1 == nodes {
                        return certificate_error(
                            "assignment residual network has a negative cycle",
                        );
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    Ok(potentials)
}

fn operations_from_flow<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    layout: &Layout<C>,
) -> Result<Vec<AssignmentOperation<C>>, GraphError> {
    let mut operations = Vec::new();
    for source in 0..costs.rows() {
        let targets = (0..costs.columns())
            .filter(|target| layout.network.flow(layout.pairs[source][*target]) == 1)
            .collect::<Vec<_>>();
        if let Some((&first, rest)) = targets.split_first() {
            operations.push(AssignmentOperation::Match {
                source,
                target: first,
                cost: costs.value(source, first).clone(),
            });
            let doubling = policy.doubling_cost(source);
            for &target in rest {
                let cost = add(
                    costs.value(source, target),
                    doubling.ok_or_else(|| {
                        GraphError::InvalidAssignment(
                            "flow doubled a source under a forbid policy".to_owned(),
                        )
                    })?,
                    "doubling operation",
                )?;
                operations.push(AssignmentOperation::Double {
                    source,
                    target,
                    cost,
                });
            }
        } else {
            operations.push(AssignmentOperation::Delete {
                source,
                cost: policy.deletion_costs[source].clone(),
            });
        }
    }
    for target in 0..costs.columns() {
        if layout.network.flow(layout.insertion[target]) == 1 {
            operations.push(AssignmentOperation::Insert {
                target,
                cost: policy.insertion_costs[target].clone(),
            });
        }
    }
    Ok(operations)
}

pub(super) fn verify<C: AssignmentCost>(
    costs: &CostMatrix<C>,
    policy: &AssignmentPolicy<C>,
    assignment: &Assignment<C>,
    potentials: &[C],
) -> Result<(), GraphError> {
    let mut layout = build(costs, policy)?;
    let mut source_counts = vec![0usize; costs.rows()];
    let mut inserted = vec![false; costs.columns()];
    for operation in &assignment.operations {
        match operation {
            AssignmentOperation::Match { source, target, .. }
            | AssignmentOperation::Double { source, target, .. } => {
                source_counts[*source] += 1;
                layout.network.send(layout.pairs[*source][*target], 1)?;
            }
            AssignmentOperation::Insert { target, .. } => inserted[*target] = true,
            AssignmentOperation::Delete { .. } => {}
        }
    }
    for (source, count) in source_counts.into_iter().enumerate() {
        if count == 0 {
            continue;
        }
        layout.network.send(layout.source_first[source], 1)?;
        if count > 1 {
            let arc = layout.source_double[source].ok_or_else(|| {
                GraphError::CertificateInvalid("doubling network arc is absent".to_owned())
            })?;
            layout.network.send(arc, count - 1)?;
        }
    }
    for (target, is_inserted) in inserted.into_iter().enumerate() {
        if is_inserted {
            layout.network.send(layout.insertion[target], 1)?;
        }
        layout.network.send(layout.target_sink[target], 1)?;
    }

    if potentials.len() != layout.network.adjacency.len() {
        return certificate_error("dual potential count does not match assignment network");
    }
    for (from, edges) in layout.network.adjacency.iter().enumerate() {
        for edge in edges {
            if edge.capacity == 0 {
                continue;
            }
            let with_source = add(&edge.cost, &potentials[from], "assignment reduced cost")?;
            let reduced = sub(
                &with_source,
                &potentials[edge.to],
                "assignment reduced cost",
            )?;
            if reduced < C::zero() {
                return certificate_error("dual potential admits a negative reduced-cost edge");
            }
        }
    }
    Ok(())
}
