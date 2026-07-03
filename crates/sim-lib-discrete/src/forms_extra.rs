use super::{FormError, Token, expect_version, ints, list_to_usize, parse_form, usizes};

/// A list of `(left, right, weight)` triples shared by sparse-matrix entries
/// (row, column, value) and graph edges (source, target, weight).
pub type WeightedTriples = Vec<(usize, usize, i64)>;
/// The decoded payload of a sparse-matrix form: `(rows, cols, entries)`.
pub type SparseMatrixParts = (usize, usize, WeightedTriples);
/// The decoded payload of a graph form: `(directed, node_count, edges)`.
pub type GraphParts = (bool, usize, WeightedTriples);

fn triples(values: &[(usize, usize, i64)]) -> String {
    values
        .iter()
        .flat_map(|(a, b, c)| [a.to_string(), b.to_string(), c.to_string()])
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_triples(values: &[i64], context: &str) -> Result<WeightedTriples, FormError> {
    if !values.len().is_multiple_of(3) {
        return Err(FormError::BadArity(format!(
            "{context} expects triples, got {} scalar(s)",
            values.len()
        )));
    }
    let mut out = Vec::with_capacity(values.len() / 3);
    for chunk in values.chunks_exact(3) {
        out.push((
            usize::try_from(chunk[0]).map_err(|_| FormError::BadToken(chunk[0].to_string()))?,
            usize::try_from(chunk[1]).map_err(|_| FormError::BadToken(chunk[1].to_string()))?,
            chunk[2],
        ));
    }
    Ok(out)
}

fn expect_head(head: &str, expected: &str) -> Result<(), FormError> {
    if head == expected {
        Ok(())
    } else {
        Err(FormError::BadShape(format!(
            "expected {expected}, got {head}"
        )))
    }
}

fn expect_non_negative(value: i64, field: &str) -> Result<usize, FormError> {
    usize::try_from(value).map_err(|_| FormError::BadToken(format!("{field}={value}")))
}

/// `#(discrete/edge v1 id source target weight)`.
pub fn encode_edge(id: usize, source: usize, target: usize, weight: i64) -> String {
    format!("#(discrete/edge v1 {id} {source} {target} {weight})")
}

/// Decode an edge form into `(id, source, target, weight)`.
pub fn decode_edge(s: &str) -> Result<(usize, usize, usize, i64), FormError> {
    let (head, tokens) = parse_form(s)?;
    expect_head(&head, "discrete/edge")?;
    expect_version(&tokens)?;
    match &tokens[1..] {
        [
            Token::Int(id),
            Token::Int(source),
            Token::Int(target),
            Token::Int(weight),
        ] => Ok((
            expect_non_negative(*id, "id")?,
            expect_non_negative(*source, "source")?,
            expect_non_negative(*target, "target")?,
            *weight,
        )),
        _ => Err(FormError::BadArity(
            "expected v1 id source target weight".to_string(),
        )),
    }
}

/// `#(discrete/sparse-matrix v1 int rows cols [row col value ...])`.
///
/// # Examples
///
/// ```
/// use sim_lib_discrete::forms::{encode_sparse_matrix, decode_sparse_matrix};
///
/// let text = encode_sparse_matrix(3, 3, &[(0, 1, 5), (2, 0, -1)]);
/// assert_eq!(
///     decode_sparse_matrix(&text).unwrap(),
///     (3, 3, vec![(0, 1, 5), (2, 0, -1)]),
/// );
/// ```
pub fn encode_sparse_matrix(rows: usize, cols: usize, entries: &[(usize, usize, i64)]) -> String {
    format!(
        "#(discrete/sparse-matrix v1 int {rows} {cols} [{}])",
        triples(entries)
    )
}

/// Decode a sparse integer matrix form into `(rows, cols, entries)`.
pub fn decode_sparse_matrix(s: &str) -> Result<SparseMatrixParts, FormError> {
    let (head, tokens) = parse_form(s)?;
    expect_head(&head, "discrete/sparse-matrix")?;
    expect_version(&tokens)?;
    match &tokens[1..] {
        [
            Token::Word(domain),
            Token::Int(rows),
            Token::Int(cols),
            Token::List(data),
        ] if domain == "int" => {
            let rows = expect_non_negative(*rows, "rows")?;
            let cols = expect_non_negative(*cols, "cols")?;
            let entries = parse_triples(data, "sparse-matrix")?;
            for (row, col, _) in &entries {
                if *row >= rows || *col >= cols {
                    return Err(FormError::BadToken(format!(
                        "sparse entry ({row},{col}) outside {rows}x{cols}"
                    )));
                }
            }
            Ok((rows, cols, entries))
        }
        _ => Err(FormError::BadArity(
            "expected v1 int rows cols [row col value ...]".to_string(),
        )),
    }
}

/// `#(discrete/graph v1 directed|undirected node-count [source target weight ...])`.
///
/// # Examples
///
/// ```
/// use sim_lib_discrete::forms::{encode_graph, decode_graph};
///
/// let text = encode_graph(false, 3, &[(0, 1, 5), (1, 2, 7)]);
/// assert_eq!(
///     decode_graph(&text).unwrap(),
///     (false, 3, vec![(0, 1, 5), (1, 2, 7)]),
/// );
/// ```
pub fn encode_graph(directed: bool, node_count: usize, edges: &[(usize, usize, i64)]) -> String {
    let kind = if directed { "directed" } else { "undirected" };
    format!(
        "#(discrete/graph v1 {kind} {node_count} [{}])",
        triples(edges)
    )
}

/// Decode a graph form into `(directed, node_count, edges)`.
pub fn decode_graph(s: &str) -> Result<GraphParts, FormError> {
    let (head, tokens) = parse_form(s)?;
    expect_head(&head, "discrete/graph")?;
    expect_version(&tokens)?;
    match &tokens[1..] {
        [Token::Word(kind), Token::Int(nodes), Token::List(data)] => {
            let directed = match kind.as_str() {
                "directed" => true,
                "undirected" => false,
                _ => {
                    return Err(FormError::BadToken(format!(
                        "graph kind must be directed or undirected, got {kind}"
                    )));
                }
            };
            let node_count = expect_non_negative(*nodes, "node-count")?;
            let edges = parse_triples(data, "graph")?;
            for (source, target, _) in &edges {
                if *source >= node_count || *target >= node_count {
                    return Err(FormError::BadToken(format!(
                        "graph edge ({source},{target}) outside node-count {node_count}"
                    )));
                }
            }
            Ok((directed, node_count, edges))
        }
        _ => Err(FormError::BadArity(
            "expected v1 directed|undirected node-count [source target weight ...]".to_string(),
        )),
    }
}

/// `#(discrete/mst-certificate v1 [edge-id ...] total-weight)`.
pub fn encode_mst_certificate(edge_ids: &[usize], total_weight: i64) -> String {
    format!(
        "#(discrete/mst-certificate v1 [{}] {total_weight})",
        usizes(edge_ids)
    )
}

/// Decode an MST-certificate form into `(edge_ids, total_weight)`.
pub fn decode_mst_certificate(s: &str) -> Result<(Vec<usize>, i64), FormError> {
    let (head, tokens) = parse_form(s)?;
    expect_head(&head, "discrete/mst-certificate")?;
    expect_version(&tokens)?;
    match &tokens[1..] {
        [Token::List(edge_ids), Token::Int(total_weight)] => {
            Ok((list_to_usize(edge_ids)?, *total_weight))
        }
        _ => Err(FormError::BadArity(
            "expected v1 [edge-id ...] total-weight".to_string(),
        )),
    }
}

/// `#(discrete/rank-space v1 kind [parameters])`.
pub fn encode_rank_space(kind: &str, parameters: &[i64]) -> String {
    format!("#(discrete/rank-space v1 {kind} [{}])", ints(parameters))
}

/// Decode a rank-space adapter form into `(kind, parameters)`.
pub fn decode_rank_space(s: &str) -> Result<(String, Vec<i64>), FormError> {
    let (head, tokens) = parse_form(s)?;
    expect_head(&head, "discrete/rank-space")?;
    expect_version(&tokens)?;
    match &tokens[1..] {
        [Token::Word(kind), Token::List(parameters)] => match kind.as_str() {
            "bit-vector" | "subset" | "combination" | "permutation" | "bounded-int-vector"
            | "simple-graph" | "fwht-signal" => Ok((kind.clone(), parameters.clone())),
            _ => Err(FormError::BadToken(format!(
                "unknown discrete rank-space kind {kind}"
            ))),
        },
        _ => Err(FormError::BadArity(
            "expected v1 kind [parameters]".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_and_sparse_forms_round_trip() {
        let sparse = encode_sparse_matrix(3, 3, &[(0, 1, 5), (2, 0, -1)]);
        assert_eq!(
            decode_sparse_matrix(&sparse).unwrap(),
            (3, 3, vec![(0, 1, 5), (2, 0, -1)])
        );

        let graph = encode_graph(false, 3, &[(0, 1, 5), (1, 2, 7)]);
        assert_eq!(
            decode_graph(&graph).unwrap(),
            (false, 3, vec![(0, 1, 5), (1, 2, 7)])
        );
    }

    #[test]
    fn edge_certificate_and_rank_space_forms_round_trip() {
        let edge = encode_edge(2, 0, 1, -5);
        assert_eq!(decode_edge(&edge).unwrap(), (2, 0, 1, -5));

        let cert = encode_mst_certificate(&[0, 3], 12);
        assert_eq!(decode_mst_certificate(&cert).unwrap(), (vec![0, 3], 12));

        let space = encode_rank_space("combination", &[6, 3]);
        assert_eq!(
            decode_rank_space(&space).unwrap(),
            ("combination".to_owned(), vec![6, 3])
        );
    }

    #[test]
    fn descriptor_bounds_are_rejected() {
        assert!(matches!(
            decode_sparse_matrix("#(discrete/sparse-matrix v1 int 2 2 [0 9 1])"),
            Err(FormError::BadToken(_))
        ));
        assert!(matches!(
            decode_graph("#(discrete/graph v1 undirected 2 [0 2 1])"),
            Err(FormError::BadToken(_))
        ));
    }
}
