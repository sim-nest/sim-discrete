//! Deterministic cookbook builders for discrete algebra recipes.

use crate::{AlgebraError, Gf2, Matrix};

/// Report produced by the semiring matrix cookbook recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemiringMatrixDemo {
    /// Matrix dimensions as `(rows, cols)`.
    pub dimensions: (usize, usize),
    /// Input matrix entries in row-major order.
    pub matrix: Vec<bool>,
    /// Result of multiplying the matrix by the semiring identity.
    pub identity_product: Vec<bool>,
    /// Result of multiplying the matrix by itself over GF(2).
    pub square: Vec<bool>,
}

/// Build the modeled semiring matrix report used by the cookbook recipe.
pub fn semiring_matrix_demo() -> Result<SemiringMatrixDemo, AlgebraError> {
    let matrix = Matrix::from_rows(vec![
        vec![Gf2(true), Gf2(false)],
        vec![Gf2(true), Gf2(true)],
    ])?;
    let identity = Matrix::<Gf2>::identity(2);
    let identity_product = matrix.matmul(&identity)?;
    let square = matrix.matmul(&matrix)?;

    Ok(SemiringMatrixDemo {
        dimensions: (matrix.rows, matrix.cols),
        matrix: bits(&matrix),
        identity_product: bits(&identity_product),
        square: bits(&square),
    })
}

fn bits(matrix: &Matrix<Gf2>) -> Vec<bool> {
    matrix.data.iter().map(|bit| bit.0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semiring_demo_uses_identity_and_gf2_multiply() {
        let demo = semiring_matrix_demo().expect("valid semiring demo");

        assert_eq!(demo.dimensions, (2, 2));
        assert_eq!(demo.identity_product, demo.matrix);
        assert_eq!(demo.square, vec![true, false, false, true]);
    }
}
