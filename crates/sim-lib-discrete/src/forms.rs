//! Read-construct textual forms for discrete values.
//!
//! These are the canonical `#(discrete/<kind> v1 ...)` round-trip forms used for
//! data reconstruction (never broad read-eval). They are implemented here as
//! self-contained string codecs so the forms have a single, tested definition;
//! the live kernel read-construct registration consumes the same grammar.

#[path = "forms_extra.rs"]
mod forms_extra;

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
enum Token {
    Word(String),
    Int(i64),
    List(Vec<i64>),
}

fn tokenize(inner: &str) -> Result<Vec<Token>, FormError> {
    let chars: Vec<char> = inner.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '[' {
            let mut j = i + 1;
            let mut body = String::new();
            while j < chars.len() && chars[j] != ']' {
                body.push(chars[j]);
                j += 1;
            }
            if j >= chars.len() {
                return Err(FormError::BadShape("unterminated list".to_string()));
            }
            let mut list = Vec::new();
            for part in body.split_whitespace() {
                list.push(
                    part.parse::<i64>()
                        .map_err(|_| FormError::BadToken(part.to_string()))?,
                );
            }
            tokens.push(Token::List(list));
            i = j + 1;
        } else {
            let mut word = String::new();
            while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '[' {
                word.push(chars[i]);
                i += 1;
            }
            match word.parse::<i64>() {
                Ok(n) => tokens.push(Token::Int(n)),
                Err(_) => tokens.push(Token::Word(word)),
            }
        }
    }
    Ok(tokens)
}

/// Split a `#(head field...)` form into its head symbol and field tokens.
fn parse_form(s: &str) -> Result<(String, Vec<Token>), FormError> {
    let s = s.trim();
    let inner = s
        .strip_prefix("#(")
        .and_then(|r| r.strip_suffix(')'))
        .ok_or_else(|| FormError::BadShape("expected #( ... )".to_string()))?;
    let mut tokens = tokenize(inner)?;
    if tokens.is_empty() {
        return Err(FormError::BadShape("empty form".to_string()));
    }
    let head = match tokens.remove(0) {
        Token::Word(w) => w,
        other => {
            return Err(FormError::BadShape(format!(
                "expected head symbol, got {other:?}"
            )));
        }
    };
    Ok((head, tokens))
}

fn expect_version(tokens: &[Token]) -> Result<(), FormError> {
    match tokens.first() {
        Some(Token::Word(v)) if v == "v1" => Ok(()),
        Some(Token::Word(v)) => Err(FormError::BadVersion {
            expected: "v1".to_string(),
            found: v.clone(),
        }),
        _ => Err(FormError::BadArity("missing version token".to_string())),
    }
}

fn list_to_usize(list: &[i64]) -> Result<Vec<usize>, FormError> {
    list.iter()
        .map(|&v| usize::try_from(v).map_err(|_| FormError::BadToken(v.to_string())))
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
        [Token::Int(n), Token::Int(k), Token::List(values)] => {
            Ok((*n as usize, *k as usize, list_to_usize(values)?))
        }
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
    fn bad_version_rejected() {
        assert!(matches!(
            decode_permutation("#(discrete/permutation v2 [0 1])"),
            Err(FormError::BadVersion { .. })
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
