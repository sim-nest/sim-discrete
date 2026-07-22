use super::*;
use sim_lib_discrete_algebra::{Matrix, SparseEntry, SparseMatrix};
use sim_lib_discrete_graph::{Directedness, Edge, Graph, IntRing, MstCertificate};
use sim_lib_discrete_rank::{
    BitVectorSpace, BoundedIntVectorSpace, CombinationSpace, FwhtSignalSpace, PermutationSpace,
    SimpleGraphSpace, SubsetSpace,
};
use sim_lib_discrete_spectral::WalshSignal;

impl MatrixDescriptor {
    /// Build the citizen from dense matrix dimensions and row-major data.
    pub fn from_parts(rows: usize, cols: usize, data: &[i64]) -> Result<Self> {
        Self::from_text(&forms::encode_matrix(rows, cols, data))
    }

    /// Decode the stored form into `(rows, cols, row-major data)`.
    pub fn parts(&self) -> Result<(usize, usize, Vec<i64>)> {
        forms::decode_matrix(&self.form).map_err(form_error)
    }

    /// Convert the citizen into an integer-ring dense matrix.
    pub fn matrix(&self) -> Result<Matrix<IntRing>> {
        let (rows, cols, data) = self.parts()?;
        Ok(Matrix {
            rows,
            cols,
            data: data.into_iter().map(IntRing).collect(),
        })
    }
}

impl SparseMatrixDescriptor {
    /// Build the citizen from sparse `(row, col, value)` entries.
    pub fn from_entries(rows: usize, cols: usize, entries: &[(usize, usize, i64)]) -> Result<Self> {
        Self::from_text(&forms::encode_sparse_matrix(rows, cols, entries))
    }

    /// Convert the citizen into an integer-ring sparse matrix.
    pub fn sparse_matrix(&self) -> Result<SparseMatrix<IntRing>> {
        let (rows, cols, entries) = forms::decode_sparse_matrix(&self.form).map_err(form_error)?;
        let entries = entries
            .into_iter()
            .map(|(row, col, value)| SparseEntry {
                row,
                col,
                value: IntRing(value),
            })
            .collect();
        SparseMatrix::from_entries(rows, cols, entries).map_err(super::discrete_algebra_error)
    }
}

impl EdgeDescriptor {
    /// Build the citizen from an integer-weighted edge.
    pub fn from_edge(edge: &Edge<i64>) -> Result<Self> {
        Self::from_text(&forms::encode_edge(
            edge.id,
            edge.source,
            edge.target,
            edge.weight,
        ))
    }

    /// Convert the citizen back into an integer-weighted edge.
    pub fn edge(&self) -> Result<Edge<i64>> {
        let (id, source, target, weight) = forms::decode_edge(&self.form).map_err(form_error)?;
        Ok(Edge {
            id,
            source,
            target,
            weight,
        })
    }
}

impl GraphDescriptor {
    /// Build the citizen from an integer-weighted graph.
    pub fn from_graph(graph: &Graph<usize, i64>) -> Result<Self> {
        let edges = graph
            .edges
            .iter()
            .map(|edge| (edge.source, edge.target, edge.weight))
            .collect::<Vec<_>>();
        Self::from_text(&forms::encode_graph(
            graph.is_directed(),
            graph.node_count(),
            &edges,
        ))
    }

    /// Convert the citizen back into an integer-weighted graph.
    pub fn graph(&self) -> Result<Graph<usize, i64>> {
        let (directed, node_count, edges) = forms::decode_graph(&self.form).map_err(form_error)?;
        let kind = if directed {
            Directedness::Directed
        } else {
            Directedness::Undirected
        };
        let mut graph = Graph::with_nodes((0..node_count).collect(), kind);
        for (source, target, weight) in edges {
            graph
                .add_edge(source, target, weight)
                .map_err(super::discrete_graph_error)?;
        }
        Ok(graph)
    }
}

impl CombinationDescriptor {
    /// Build the citizen from a `k`-subset of `{0, ..., n-1}`.
    pub fn from_values(n: usize, k: usize, values: &[usize]) -> Result<Self> {
        Self::from_text(&forms::encode_combination(n, k, values))
    }

    /// Decode the stored form into `(n, k, member values)`.
    pub fn values(&self) -> Result<(usize, usize, Vec<usize>)> {
        forms::decode_combination(&self.form).map_err(form_error)
    }
}

impl PermutationDescriptor {
    /// Build the citizen from a permutation in one-line notation.
    pub fn from_values(values: &[usize]) -> Result<Self> {
        Self::from_text(&forms::encode_permutation(values))
    }

    /// Decode the stored form into the permutation values.
    pub fn values(&self) -> Result<Vec<usize>> {
        forms::decode_permutation(&self.form).map_err(form_error)
    }
}

impl FwhtSignalDescriptor {
    /// Build the citizen from natural-order signal coefficients.
    pub fn from_coeffs(values: &[i64]) -> Result<Self> {
        Self::from_text(&forms::encode_fwht_signal(values))
    }

    /// Convert the citizen into a natural-order Walsh signal.
    pub fn signal(&self) -> Result<WalshSignal<i64>> {
        Ok(WalshSignal::natural(
            forms::decode_fwht_signal(&self.form).map_err(form_error)?,
        ))
    }
}

impl MstCertificateDescriptor {
    /// Build the citizen from a minimum-spanning-tree certificate.
    pub fn from_certificate(certificate: &MstCertificate) -> Result<Self> {
        let total_weight = certificate
            .total_weight_repr
            .parse::<i64>()
            .map_err(|err| Error::Eval(format!("invalid MST total weight: {err}")))?;
        Self::from_text(&forms::encode_mst_certificate(
            &certificate.edge_ids,
            total_weight,
        ))
    }

    /// Convert the citizen back into a minimum-spanning-tree certificate.
    pub fn certificate(&self) -> Result<MstCertificate> {
        let (edge_ids, total_weight) =
            forms::decode_mst_certificate(&self.form).map_err(form_error)?;
        Ok(MstCertificate {
            edge_ids,
            total_weight_repr: total_weight.to_string(),
        })
    }
}

impl BitVectorSpaceDescriptor {
    /// Reconstruct the bit-vector rank space from the descriptor form.
    pub fn space(&self) -> Result<BitVectorSpace> {
        BitVectorSpace::try_new(rank_usize(&self.form, "bit-vector", 1)?[0])
            .map_err(super::discrete_rank_error)
    }
}

impl SubsetSpaceDescriptor {
    /// Reconstruct the subset rank space from the descriptor form.
    pub fn space(&self) -> Result<SubsetSpace> {
        SubsetSpace::try_new(rank_usize(&self.form, "subset", 1)?[0])
            .map_err(super::discrete_rank_error)
    }
}

impl CombinationSpaceDescriptor {
    /// Reconstruct the combination rank space from the descriptor form.
    pub fn space(&self) -> Result<CombinationSpace> {
        let params = rank_usize(&self.form, "combination", 2)?;
        CombinationSpace::try_new(params[0], params[1]).map_err(super::discrete_rank_error)
    }
}

impl PermutationSpaceDescriptor {
    /// Reconstruct the permutation rank space from the descriptor form.
    pub fn space(&self) -> Result<PermutationSpace> {
        PermutationSpace::try_new(rank_usize(&self.form, "permutation", 1)?[0])
            .map_err(super::discrete_rank_error)
    }
}

impl BoundedIntVectorSpaceDescriptor {
    /// Reconstruct the bounded-int-vector rank space from the descriptor form.
    pub fn space(&self) -> Result<BoundedIntVectorSpace> {
        BoundedIntVectorSpace::try_new(rank_u64(&self.form, "bounded-int-vector", None)?)
            .map_err(super::discrete_rank_error)
    }
}

impl SimpleGraphSpaceDescriptor {
    /// Reconstruct the simple-graph rank space from the descriptor form.
    pub fn space(&self) -> Result<SimpleGraphSpace> {
        SimpleGraphSpace::try_new(rank_usize(&self.form, "simple-graph", 1)?[0])
            .map_err(super::discrete_rank_error)
    }
}

impl FwhtSignalSpaceDescriptor {
    /// Reconstruct the FWHT-signal rank space from the descriptor form.
    pub fn space(&self) -> Result<FwhtSignalSpace> {
        let params = rank_u64(&self.form, "fwht-signal", Some(2))?;
        let len = usize::try_from(params[0])
            .map_err(|_| Error::Eval("FWHT signal length is out of range".to_owned()))?;
        FwhtSignalSpace::try_new(len, params[1]).map_err(super::discrete_rank_error)
    }
}

fn rank_usize(form: &str, expected: &str, arity: usize) -> Result<Vec<usize>> {
    rank_i64(form, expected, Some(arity))?
        .into_iter()
        .map(|param| {
            usize::try_from(param)
                .map_err(|_| Error::Eval(format!("rank-space parameter {param} is out of range")))
        })
        .collect()
}

fn rank_u64(form: &str, expected: &str, arity: Option<usize>) -> Result<Vec<u64>> {
    rank_i64(form, expected, arity)?
        .into_iter()
        .map(|param| {
            u64::try_from(param)
                .map_err(|_| Error::Eval(format!("rank-space parameter {param} is out of range")))
        })
        .collect()
}

fn rank_i64(form: &str, expected: &str, arity: Option<usize>) -> Result<Vec<i64>> {
    let canonical = canonical_rank_space(form, expected, arity)?;
    let (_, params) = forms::decode_rank_space(&canonical).map_err(form_error)?;
    Ok(params)
}
