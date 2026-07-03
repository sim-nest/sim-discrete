use sim_citizen_derive::Citizen;
use sim_kernel::{Error, Expr, Result, Symbol};

use crate::forms;

#[path = "citizen_values.rs"]
mod citizen_values;

macro_rules! form_citizen {
    (
        $name:ident,
        $symbol:literal,
        $symbol_fn:ident,
        $field_mod:ident,
        $field_path:literal,
        $default:expr,
        $canonical:ident
    ) => {
        #[doc = concat!("Citizen wrapper around the canonical `", $symbol, "` read-construct form.")]
        #[derive(Clone, Debug, PartialEq, Citizen)]
        #[citizen(symbol = $symbol, version = 1)]
        pub struct $name {
            #[citizen(with = $field_path)]
            form: String,
        }

        impl $name {
            /// Build the citizen from form text, canonicalizing on the way in.
            pub fn from_text(value: &str) -> Result<Self> {
                Ok(Self {
                    form: $canonical(value)?,
                })
            }

            /// Borrow the stored canonical form text.
            pub fn as_text(&self) -> &str {
                &self.form
            }

            /// Canonicalize form text and build the read-construct `Expr` directly.
            pub fn read_construct_expr_from_text(value: &str) -> Result<Expr> {
                Ok(read_construct_expr($symbol_fn(), $canonical(value)?))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                let default_form: String = $default;
                Self::from_text(&default_form)
                    .expect("default discrete citizen form should be valid")
            }
        }

        #[doc = concat!("The class symbol `", $symbol, "` for this citizen.")]
        pub fn $symbol_fn() -> Symbol {
            sim_citizen::parse_symbol($symbol)
        }

        pub(crate) mod $field_mod {
            use sim_kernel::{Error, Expr, Result};

            pub fn encode(value: &str) -> Expr {
                Expr::String(value.to_owned())
            }

            pub fn decode(expr: &Expr) -> Result<String> {
                match expr {
                    Expr::String(value) => super::$canonical(value),
                    other => Err(Error::Eval(format!(
                        "discrete citizen form must be a string, found {}",
                        super::expr_kind(other)
                    ))),
                }
            }
        }
    };
}

form_citizen!(
    MatrixDescriptor,
    "discrete/Matrix",
    discrete_matrix_class_symbol,
    matrix_form,
    "matrix_form",
    forms::encode_matrix(2, 2, &[1, 2, 3, 4]),
    canonical_matrix
);
form_citizen!(
    SparseMatrixDescriptor,
    "discrete/SparseMatrix",
    discrete_sparse_matrix_class_symbol,
    sparse_matrix_form,
    "sparse_matrix_form",
    forms::encode_sparse_matrix(2, 2, &[(0, 1, 5)]),
    canonical_sparse_matrix
);
form_citizen!(
    GraphDescriptor,
    "discrete/Graph",
    discrete_graph_class_symbol,
    graph_form,
    "graph_form",
    forms::encode_graph(false, 3, &[(0, 1, 1), (1, 2, 2)]),
    canonical_graph
);
form_citizen!(
    EdgeDescriptor,
    "discrete/Edge",
    discrete_edge_class_symbol,
    edge_form,
    "edge_form",
    forms::encode_edge(0, 0, 1, 1),
    canonical_edge
);
form_citizen!(
    CombinationDescriptor,
    "discrete/Combination",
    discrete_combination_class_symbol,
    combination_form,
    "combination_form",
    forms::encode_combination(5, 3, &[0, 2, 4]),
    canonical_combination
);
form_citizen!(
    PermutationDescriptor,
    "discrete/Permutation",
    discrete_permutation_class_symbol,
    permutation_form,
    "permutation_form",
    forms::encode_permutation(&[2, 0, 1]),
    canonical_permutation
);
form_citizen!(
    FwhtSignalDescriptor,
    "discrete/FwhtSignal",
    discrete_fwht_signal_class_symbol,
    fwht_signal_form,
    "fwht_signal_form",
    forms::encode_fwht_signal(&[1, 0, 0, 0]),
    canonical_fwht_signal
);
form_citizen!(
    MstCertificateDescriptor,
    "discrete/MstCertificate",
    discrete_mst_certificate_class_symbol,
    mst_certificate_form,
    "mst_certificate_form",
    forms::encode_mst_certificate(&[0, 1], 3),
    canonical_mst_certificate
);
form_citizen!(
    BitVectorSpaceDescriptor,
    "discrete/BitVectorSpace",
    discrete_bit_vector_space_class_symbol,
    bit_vector_space_form,
    "bit_vector_space_form",
    forms::encode_rank_space("bit-vector", &[4]),
    canonical_bit_vector_space
);
form_citizen!(
    SubsetSpaceDescriptor,
    "discrete/SubsetSpace",
    discrete_subset_space_class_symbol,
    subset_space_form,
    "subset_space_form",
    forms::encode_rank_space("subset", &[5]),
    canonical_subset_space
);
form_citizen!(
    CombinationSpaceDescriptor,
    "discrete/CombinationSpace",
    discrete_combination_space_class_symbol,
    combination_space_form,
    "combination_space_form",
    forms::encode_rank_space("combination", &[6, 3]),
    canonical_combination_space
);
form_citizen!(
    PermutationSpaceDescriptor,
    "discrete/PermutationSpace",
    discrete_permutation_space_class_symbol,
    permutation_space_form,
    "permutation_space_form",
    forms::encode_rank_space("permutation", &[4]),
    canonical_permutation_space
);
form_citizen!(
    BoundedIntVectorSpaceDescriptor,
    "discrete/BoundedIntVectorSpace",
    discrete_bounded_int_vector_space_class_symbol,
    bounded_int_vector_space_form,
    "bounded_int_vector_space_form",
    forms::encode_rank_space("bounded-int-vector", &[3, 2, 4]),
    canonical_bounded_int_vector_space
);
form_citizen!(
    SimpleGraphSpaceDescriptor,
    "discrete/SimpleGraphSpace",
    discrete_simple_graph_space_class_symbol,
    simple_graph_space_form,
    "simple_graph_space_form",
    forms::encode_rank_space("simple-graph", &[4]),
    canonical_simple_graph_space
);
form_citizen!(
    FwhtSignalSpaceDescriptor,
    "discrete/FwhtSignalSpace",
    discrete_fwht_signal_space_class_symbol,
    fwht_signal_space_form,
    "fwht_signal_space_form",
    forms::encode_rank_space("fwht-signal", &[3, 2]),
    canonical_fwht_signal_space
);

fn canonical_matrix(value: &str) -> Result<String> {
    forms::decode_matrix(value)
        .map(|(rows, cols, data)| forms::encode_matrix(rows, cols, &data))
        .map_err(form_error)
}

fn canonical_sparse_matrix(value: &str) -> Result<String> {
    forms::decode_sparse_matrix(value)
        .map(|(rows, cols, entries)| forms::encode_sparse_matrix(rows, cols, &entries))
        .map_err(form_error)
}

fn canonical_graph(value: &str) -> Result<String> {
    forms::decode_graph(value)
        .map(|(directed, nodes, edges)| forms::encode_graph(directed, nodes, &edges))
        .map_err(form_error)
}

fn canonical_edge(value: &str) -> Result<String> {
    forms::decode_edge(value)
        .map(|(id, source, target, weight)| forms::encode_edge(id, source, target, weight))
        .map_err(form_error)
}

fn canonical_combination(value: &str) -> Result<String> {
    let (n, k, values) = forms::decode_combination(value).map_err(form_error)?;
    sim_lib_discrete_comb::combination_rank(&values, n).map_err(discrete_comb_error)?;
    if values.len() != k {
        return Err(Error::Eval(format!(
            "combination length {} != k {k}",
            values.len()
        )));
    }
    Ok(forms::encode_combination(n, k, &values))
}

fn canonical_permutation(value: &str) -> Result<String> {
    let values = forms::decode_permutation(value).map_err(form_error)?;
    sim_lib_discrete_comb::permutation_rank(&values).map_err(discrete_comb_error)?;
    Ok(forms::encode_permutation(&values))
}

fn canonical_fwht_signal(value: &str) -> Result<String> {
    forms::decode_fwht_signal(value)
        .map(|values| forms::encode_fwht_signal(&values))
        .map_err(form_error)
}

fn canonical_mst_certificate(value: &str) -> Result<String> {
    forms::decode_mst_certificate(value)
        .map(|(edge_ids, total_weight)| forms::encode_mst_certificate(&edge_ids, total_weight))
        .map_err(form_error)
}

fn canonical_bit_vector_space(value: &str) -> Result<String> {
    canonical_rank_space(value, "bit-vector", Some(1))
}

fn canonical_subset_space(value: &str) -> Result<String> {
    canonical_rank_space(value, "subset", Some(1))
}

fn canonical_combination_space(value: &str) -> Result<String> {
    canonical_rank_space(value, "combination", Some(2))
}

fn canonical_permutation_space(value: &str) -> Result<String> {
    canonical_rank_space(value, "permutation", Some(1))
}

fn canonical_bounded_int_vector_space(value: &str) -> Result<String> {
    canonical_rank_space(value, "bounded-int-vector", None)
}

fn canonical_simple_graph_space(value: &str) -> Result<String> {
    canonical_rank_space(value, "simple-graph", Some(1))
}

fn canonical_fwht_signal_space(value: &str) -> Result<String> {
    canonical_rank_space(value, "fwht-signal", Some(2))
}

fn canonical_rank_space(value: &str, expected: &str, arity: Option<usize>) -> Result<String> {
    let (kind, params) = forms::decode_rank_space(value).map_err(form_error)?;
    if kind != expected {
        return Err(Error::Eval(format!(
            "expected rank-space kind {expected}, got {kind}"
        )));
    }
    if let Some(expected_len) = arity
        && params.len() != expected_len
    {
        return Err(Error::Eval(format!(
            "rank-space {kind} expects {expected_len} parameter(s), got {}",
            params.len()
        )));
    }
    if params.iter().any(|param| *param < 0) {
        return Err(Error::Eval(
            "rank-space parameters must be non-negative".to_owned(),
        ));
    }
    Ok(forms::encode_rank_space(&kind, &params))
}

fn read_construct_expr(class: Symbol, form: String) -> Expr {
    Expr::Extension {
        tag: Symbol::qualified("citizen", "read-construct"),
        payload: Box::new(Expr::Vector(vec![
            Expr::Symbol(class),
            Expr::Symbol(Symbol::new("v1")),
            Expr::String(form),
        ])),
    }
}

fn form_error(error: forms::FormError) -> Error {
    Error::domain_error(
        Symbol::new("discrete"),
        Symbol::qualified("discrete", "citizen-form"),
        error.to_string(),
    )
}

fn discrete_comb_error(error: sim_lib_discrete_comb::CombError) -> Error {
    Error::domain_error(
        Symbol::new("discrete"),
        Symbol::qualified("discrete", "comb"),
        error.to_string(),
    )
}

fn discrete_algebra_error(error: sim_lib_discrete_algebra::AlgebraError) -> Error {
    Error::domain_error(
        Symbol::new("discrete"),
        Symbol::qualified("discrete", "algebra"),
        error.to_string(),
    )
}

fn discrete_graph_error(error: sim_lib_discrete_graph::GraphError) -> Error {
    Error::domain_error(
        Symbol::new("discrete"),
        Symbol::qualified("discrete", "graph"),
        error.to_string(),
    )
}

fn expr_kind(expr: &Expr) -> &'static str {
    match expr {
        Expr::Nil => "nil",
        Expr::Bool(_) => "bool",
        Expr::Number(_) => "number",
        Expr::Symbol(_) => "symbol",
        Expr::Local(_) => "local",
        Expr::String(_) => "string",
        Expr::Bytes(_) => "bytes",
        Expr::List(_) => "list",
        Expr::Vector(_) => "vector",
        Expr::Map(_) => "map",
        Expr::Set(_) => "set",
        Expr::Call { .. } => "call",
        Expr::Infix { .. } => "infix",
        Expr::Extension { .. } => "extension",
        _ => "expr",
    }
}
