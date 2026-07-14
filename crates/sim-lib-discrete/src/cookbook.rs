//! Deterministic cookbook builders for discrete facade recipes.

use sim_kernel::{Expr, NumberLiteral, Symbol};

use crate::forms;

/// Build the modeled matrix runtime descriptor used by the cookbook recipe.
pub fn matrix_runtime_demo() -> Expr {
    let form = forms::encode_matrix(2, 2, &[1, 2, 3, 4]);
    let (_, _, data) = forms::decode_matrix(&form).expect("valid cookbook matrix form");
    let determinant = data[0] * data[3] - data[1] * data[2];

    Expr::Map(vec![
        (field("kind"), sym("discrete", "matrix-runtime")),
        (field("form"), Expr::String(form)),
        (field("rows"), number(2)),
        (field("cols"), number(2)),
        (field("determinant"), number(determinant)),
    ])
}

fn field(name: &str) -> Expr {
    Expr::Symbol(Symbol::qualified("discrete", name))
}

fn sym(namespace: &str, name: &str) -> Expr {
    Expr::Symbol(Symbol::qualified(namespace, name))
}

fn number(value: impl ToString) -> Expr {
    Expr::Number(NumberLiteral {
        domain: Symbol::qualified("numbers", "i64"),
        canonical: value.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_runtime_demo_uses_canonical_matrix_form() {
        let Expr::Map(entries) = matrix_runtime_demo() else {
            panic!("matrix runtime demo is a map")
        };

        assert!(entries.iter().any(|(key, value)| matches!(
            (key, value),
            (Expr::Symbol(symbol), Expr::Number(number))
                if symbol.name.as_ref() == "determinant" && number.canonical == "-2"
        )));
        assert!(entries.iter().any(|(key, value)| matches!(
            (key, value),
            (Expr::Symbol(symbol), Expr::String(form))
                if symbol.name.as_ref() == "form" && form.contains("discrete/matrix")
        )));
    }
}
