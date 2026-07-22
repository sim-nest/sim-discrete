//! Read-construct textual forms for discrete values.
//!
//! These are the canonical `#(discrete/<kind> v1 ...)` round-trip forms used for
//! data reconstruction (never broad read-eval). They are implemented here as
//! self-contained string codecs so the forms have a single, tested definition;
//! the live kernel read-construct registration consumes the same grammar.

// conformance: discrete algebra values round-trip through domain forms.

#[path = "forms_extra.rs"]
mod forms_extra;

use sim_codec::{DomainFormError, DomainValue, parse_domain_form};

pub use forms_extra::*;

/// Errors from parsing a read-construct form.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FormError {
    /// The overall `#(...)` shape was malformed.
    #[error("malformed form: {0}")]
    BadShape(String),
    /// The wrong number or kind of fields for this form.
    #[error("bad arity: {0}")]
    BadArity(String),
    /// The version token did not match the expected `v1`.
    #[error("bad version: expected {expected}, found {found}")]
    BadVersion {
        /// Expected version token.
        expected: String,
        /// Found token.
        found: String,
    },
    /// A token could not be parsed.
    #[error("bad token: {0}")]
    BadToken(String),
}

/// A parsed token: a bare word, an integer, or a bracketed integer list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    Word(String),
    Int(i64),
    List(Vec<i64>),
}

/// Split a `#(head field...)` form into its head symbol and field tokens.
pub(crate) fn parse_form(s: &str) -> Result<(String, Vec<Token>), FormError> {
    let normalized = normalize_discrete_form(s)?;
    let form = parse_domain_form(&normalized).map_err(domain_form_error)?;
    if !form.fields.is_empty() {
        return Err(FormError::BadArity(
            "discrete forms use positional fields".to_string(),
        ));
    }
    let head = denormalize_discrete_head(form.name);
    let tokens = form
        .positional
        .iter()
        .map(token_from_domain_value)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((head, tokens))
}

pub(crate) fn expect_version(tokens: &[Token]) -> Result<(), FormError> {
    match tokens.first() {
        Some(Token::Word(v)) if v == "v1" => Ok(()),
        Some(Token::Word(v)) => Err(FormError::BadVersion {
            expected: "v1".to_string(),
            found: v.clone(),
        }),
        _ => Err(FormError::BadArity("missing version token".to_string())),
    }
}

fn normalize_discrete_form(s: &str) -> Result<String, FormError> {
    let normalized_head = normalize_discrete_head(s.trim());
    normalize_discrete_int_lists(&normalized_head)
}

fn normalize_discrete_head(s: &str) -> String {
    let Some(rest) = s.strip_prefix("#(discrete/") else {
        return s.to_string();
    };
    // DomainForm names are identifiers, while these shipped read-construct
    // heads are namespaced symbols. Translate only the discrete head before
    // parsing, then restore the public name after parsing.
    let split = rest
        .find(|ch: char| ch.is_whitespace() || ch == ')')
        .unwrap_or(rest.len());
    format!("#(discrete-{}{}", &rest[..split], &rest[split..])
}

fn denormalize_discrete_head(name: String) -> String {
    name.strip_prefix("discrete-")
        .map(|kind| format!("discrete/{kind}"))
        .unwrap_or(name)
}

fn normalize_discrete_int_lists(s: &str) -> Result<String, FormError> {
    let chars = s.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '[' {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        // DomainForm uses comma-separated lists. Discrete's public forms
        // already use whitespace-separated integer vectors, so this is the
        // only domain-specific syntax bridge kept in this module.
        out.push('[');
        i += 1;
        let mut first = true;
        loop {
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i >= chars.len() {
                return Err(FormError::BadShape("unterminated list".to_string()));
            }
            if chars[i] == ']' {
                out.push(']');
                i += 1;
                break;
            }
            if chars[i] == ',' {
                return Err(FormError::BadToken(",".to_string()));
            }
            if !first {
                out.push(',');
            }
            first = false;

            let start = i;
            while i < chars.len() && !chars[i].is_whitespace() && chars[i] != ',' && chars[i] != ']'
            {
                i += 1;
            }
            if start == i {
                return Err(FormError::BadToken(chars[i].to_string()));
            }
            out.extend(chars[start..i].iter());

            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i < chars.len() && chars[i] == ',' {
                i += 1;
            }
        }
    }
    Ok(out)
}

fn domain_form_error(error: DomainFormError) -> FormError {
    match error {
        DomainFormError::ExpectedForm => FormError::BadShape("expected #( ... )".to_string()),
        DomainFormError::UnexpectedEof => FormError::BadShape("unexpected end of form".to_string()),
        DomainFormError::InvalidToken => {
            FormError::BadShape("invalid domain form token".to_string())
        }
        DomainFormError::DuplicateField(field) => {
            FormError::BadArity(format!("duplicate field {field}"))
        }
        DomainFormError::TrailingInput => FormError::BadShape("trailing input".to_string()),
        DomainFormError::MissingField(field) => FormError::BadToken(format!("missing {field}")),
        DomainFormError::WrongFieldKind(field) => FormError::BadToken(format!("bad {field}")),
        DomainFormError::WrongValueKind => FormError::BadToken("bad value kind".to_string()),
    }
}

fn token_from_domain_value(value: &DomainValue) -> Result<Token, FormError> {
    match value {
        DomainValue::Atom(text) => match text.parse::<i64>() {
            Ok(n) => Ok(Token::Int(n)),
            Err(_) => Ok(Token::Word(text.clone())),
        },
        DomainValue::List(items) => items
            .iter()
            .map(int_from_domain_value)
            .collect::<Result<Vec<_>, _>>()
            .map(Token::List),
        DomainValue::String(_) | DomainValue::Form(_) => {
            Err(FormError::BadToken(value.render_text()))
        }
    }
}

fn int_from_domain_value(value: &DomainValue) -> Result<i64, FormError> {
    match value {
        DomainValue::Atom(text) => text
            .parse::<i64>()
            .map_err(|_| FormError::BadToken(text.clone())),
        _ => Err(FormError::BadToken(value.render_text())),
    }
}

pub(crate) fn non_negative_usize(value: i64, field: &str) -> Result<usize, FormError> {
    usize::try_from(value).map_err(|_| FormError::BadToken(format!("{field}={value}")))
}

fn list_to_usize(list: &[i64]) -> Result<Vec<usize>, FormError> {
    list.iter()
        .map(|&v| non_negative_usize(v, "list item"))
        .collect()
}

fn ints(values: &[i64]) -> String {
    values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn usizes(values: &[usize]) -> String {
    values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// `#(discrete/combination v1 n k [values])`.
pub fn encode_combination(n: usize, k: usize, values: &[usize]) -> String {
    format!("#(discrete/combination v1 {n} {k} [{}])", usizes(values))
}

/// Decode a combination form into `(n, k, values)`.
pub fn decode_combination(s: &str) -> Result<(usize, usize, Vec<usize>), FormError> {
    let (head, tokens) = parse_form(s)?;
    if head != "discrete/combination" {
        return Err(FormError::BadShape(format!("not a combination: {head}")));
    }
    expect_version(&tokens)?;
    match &tokens[1..] {
        [Token::Int(n), Token::Int(k), Token::List(values)] => Ok((
            non_negative_usize(*n, "n")?,
            non_negative_usize(*k, "k")?,
            list_to_usize(values)?,
        )),
        _ => Err(FormError::BadArity("expected v1 n k [values]".to_string())),
    }
}

/// `#(discrete/permutation v1 [values])`.
pub fn encode_permutation(values: &[usize]) -> String {
    format!("#(discrete/permutation v1 [{}])", usizes(values))
}

/// Decode a permutation form into its value list.
pub fn decode_permutation(s: &str) -> Result<Vec<usize>, FormError> {
    let (head, tokens) = parse_form(s)?;
    if head != "discrete/permutation" {
        return Err(FormError::BadShape(format!("not a permutation: {head}")));
    }
    expect_version(&tokens)?;
    match &tokens[1..] {
        [Token::List(values)] => list_to_usize(values),
        _ => Err(FormError::BadArity("expected v1 [values]".to_string())),
    }
}

/// `#(discrete/fwht-signal v1 natural none [coeffs])`.
pub fn encode_fwht_signal(coeffs: &[i64]) -> String {
    format!("#(discrete/fwht-signal v1 natural none [{}])", ints(coeffs))
}

/// Decode an FWHT-signal form into its coefficient vector.
pub fn decode_fwht_signal(s: &str) -> Result<Vec<i64>, FormError> {
    let (head, tokens) = parse_form(s)?;
    if head != "discrete/fwht-signal" {
        return Err(FormError::BadShape(format!("not an fwht-signal: {head}")));
    }
    expect_version(&tokens)?;
    match &tokens[1..] {
        [Token::Word(basis), Token::Word(norm), Token::List(coeffs)]
            if basis == "natural" && norm == "none" =>
        {
            Ok(coeffs.clone())
        }
        _ => Err(FormError::BadArity(
            "expected v1 natural none [coeffs]".to_string(),
        )),
    }
}

/// `#(discrete/matrix v1 int rows cols [data])` (row-major).
pub fn encode_matrix(rows: usize, cols: usize, data: &[i64]) -> String {
    format!("#(discrete/matrix v1 int {rows} {cols} [{}])", ints(data))
}

/// Decode a dense integer matrix form into `(rows, cols, data)`, checking that
/// `data.len() == rows * cols`.
pub fn decode_matrix(s: &str) -> Result<(usize, usize, Vec<i64>), FormError> {
    let (head, tokens) = parse_form(s)?;
    if head != "discrete/matrix" {
        return Err(FormError::BadShape(format!("not a matrix: {head}")));
    }
    expect_version(&tokens)?;
    match &tokens[1..] {
        [
            Token::Word(domain),
            Token::Int(rows),
            Token::Int(cols),
            Token::List(data),
        ] if domain == "int" => {
            // A negative dimension would cast to ~usize::MAX and `r * c` could
            // overflow, so validate both before any size comparison.
            let r = usize::try_from(*rows)
                .map_err(|_| FormError::BadToken(format!("negative rows: {rows}")))?;
            let c = usize::try_from(*cols)
                .map_err(|_| FormError::BadToken(format!("negative cols: {cols}")))?;
            let expected = r.checked_mul(c).ok_or_else(|| {
                FormError::BadArity(format!("matrix dimensions {r}*{c} overflow"))
            })?;
            if data.len() != expected {
                return Err(FormError::BadArity(format!(
                    "matrix data length {} != {r}*{c}",
                    data.len()
                )));
            }
            Ok((r, c, data.clone()))
        }
        _ => Err(FormError::BadArity(
            "expected v1 int rows cols [data]".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combination_round_trips() {
        let s = encode_combination(6, 3, &[0, 2, 5]);
        assert_eq!(decode_combination(&s).unwrap(), (6, 3, vec![0, 2, 5]));
    }

    #[test]
    fn permutation_round_trips() {
        let s = encode_permutation(&[2, 0, 1]);
        assert_eq!(decode_permutation(&s).unwrap(), vec![2, 0, 1]);
    }

    #[test]
    fn domain_form_comma_lists_decode() {
        assert_eq!(
            decode_permutation("#(discrete/permutation v1 [2,0,1])").unwrap(),
            vec![2, 0, 1]
        );
    }

    #[test]
    fn fwht_signal_round_trips() {
        let s = encode_fwht_signal(&[1, -2, 3, 0]);
        assert_eq!(decode_fwht_signal(&s).unwrap(), vec![1, -2, 3, 0]);
    }

    #[test]
    fn matrix_round_trips() {
        let s = encode_matrix(2, 3, &[1, 2, 3, 4, 5, 6]);
        assert_eq!(decode_matrix(&s).unwrap(), (2, 3, vec![1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn bad_arity_rejected() {
        // Combination missing k.
        assert!(matches!(
            decode_combination("#(discrete/combination v1 6 [0 1])"),
            Err(FormError::BadArity(_))
        ));
        // Matrix with wrong data length.
        assert!(matches!(
            decode_matrix("#(discrete/matrix v1 int 2 2 [1 2 3])"),
            Err(FormError::BadArity(_))
        ));
    }

    #[test]
    fn negative_or_huge_matrix_dimensions_rejected() {
        // A negative declared dimension must error before allocation, not cast to
        // a near-usize::MAX length.
        assert!(matches!(
            decode_matrix("#(discrete/matrix v1 int -1 3 [1 2 3])"),
            Err(FormError::BadToken(_))
        ));
        // A pair whose product overflows usize must error rather than wrap.
        let huge = format!("#(discrete/matrix v1 int {max} {max} [1])", max = i64::MAX);
        assert!(matches!(
            decode_matrix(&huge),
            Err(FormError::BadToken(_) | FormError::BadArity(_))
        ));
    }

    #[test]
    fn negative_combination_dimensions_rejected() {
        assert!(matches!(
            decode_combination("#(discrete/combination v1 -1 1 [0])"),
            Err(FormError::BadToken(_))
        ));
        assert!(matches!(
            decode_combination("#(discrete/combination v1 3 -1 [0])"),
            Err(FormError::BadToken(_))
        ));
    }

    #[test]
    fn bad_version_rejected() {
        assert!(matches!(
            decode_permutation("#(discrete/permutation v2 [0 1])"),
            Err(FormError::BadVersion { .. })
        ));
        assert!(matches!(
            decode_permutation("#(discrete/permutation 1 [0 1])"),
            Err(FormError::BadArity(_))
        ));
    }

    #[test]
    fn malformed_shape_rejected() {
        assert!(matches!(
            decode_matrix("discrete/matrix v1 int 1 1 [1]"),
            Err(FormError::BadShape(_))
        ));
        assert!(matches!(
            decode_combination("#(discrete/permutation v1 [0])"),
            Err(FormError::BadShape(_))
        ));
    }
}
