//! Kleene closure `A* = I + A + A^2 + ...` over a closed semiring.
//!
//! This is the single derivation point for several graph algorithms:
//! boolean closure yields transitive reachability, min-plus closure yields
//! all-pairs shortest paths, and so on. The graph crate wraps this; it never
//! re-implements Floyd-Warshall or Warshall directly.

use crate::error::AlgebraError;
use crate::matrix::{AlgebraLimits, Matrix};
use crate::semiring::Semiring;

impl<S: Semiring> Matrix<S> {
    /// Compute the Kleene closure `A* = I + A + A^2 + ...`.
    ///
    /// Uses the generalized Floyd-Warshall (Lehmann) asteration: for each
    /// pivot `k`, paths may pass through `k` and loop there `star(A[k][k])`
    /// times. The identity is added at the end so `A*` includes the empty path
    /// on the diagonal (distance-0 for min-plus, reflexive for boolean).
    ///
    /// Returns [`AlgebraError::NoStar`] when a pivot's diagonal star does not
    /// converge (e.g. a negative cycle in min-plus, any directed cycle in
    /// counting, or a semiring with no `star` such as `RealF64`).
    ///
    /// # Examples
    ///
    /// Boolean closure of a directed chain `0 -> 1 -> 2` yields reflexive
    /// reachability: node `i` reaches node `j` exactly when `j >= i`.
    ///
    /// ```
    /// use sim_lib_discrete_algebra::{AlgebraLimits, BoolRing, Matrix};
    ///
    /// let mut a = Matrix::new(3, 3);
    /// a.set(0, 1, BoolRing(true)).unwrap();
    /// a.set(1, 2, BoolRing(true)).unwrap();
    ///
    /// let reach = a.closure(AlgebraLimits::default()).unwrap();
    /// assert_eq!(reach.get(0, 2).unwrap(), &BoolRing(true)); // 0 reaches 2
    /// assert_eq!(reach.get(1, 1).unwrap(), &BoolRing(true)); // reflexive
    /// assert_eq!(reach.get(2, 0).unwrap(), &BoolRing(false)); // 2 cannot reach 0
    /// ```
    pub fn closure(&self, limits: AlgebraLimits) -> Result<Self, AlgebraError> {
        self.validate()?;
        if !self.is_square() {
            return Err(AlgebraError::ShapeMismatch(format!(
                "closure requires a square matrix, got {}x{}",
                self.rows, self.cols
            )));
        }
        let n = self.rows;
        if n > limits.max_dim {
            return Err(AlgebraError::LimitExceeded(format!(
                "closure: dimension {n} exceeds max_dim {}",
                limits.max_dim
            )));
        }
        let mut c = self.clone();
        for k in 0..n {
            let s = c.data[k * n + k].star().ok_or(AlgebraError::NoStar)?;
            let mut next = c.clone();
            for i in 0..n {
                let cik = c.data[i * n + k].mul(&s);
                for j in 0..n {
                    let via = cik.mul(&c.data[k * n + j]);
                    next.data[i * n + j] = c.data[i * n + j].add(&via);
                }
            }
            c = next;
        }
        // Add the identity so the closure contains the empty path A^0 = I.
        for i in 0..n {
            let updated = c.data[i * n + i].add(&S::one());
            c.data[i * n + i] = updated;
        }
        Ok(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boolean::BoolRing;
    use crate::tropical_min::MinPlus;

    #[test]
    fn boolean_closure_of_directed_chain_is_upper_triangular_reachability() {
        // Directed chain 0->1->2->3. Reachability is the reflexive upper
        // triangle: i reaches j iff j >= i.
        let mut a = Matrix::new(4, 4);
        a.set(0, 1, BoolRing(true)).unwrap();
        a.set(1, 2, BoolRing(true)).unwrap();
        a.set(2, 3, BoolRing(true)).unwrap();
        let star = a.closure(AlgebraLimits::default()).unwrap();
        for i in 0..4 {
            for j in 0..4 {
                let expected = BoolRing(j >= i);
                assert_eq!(star.get(i, j).unwrap(), &expected, "reachability ({i},{j})");
            }
        }
    }

    #[test]
    fn min_plus_closure_matches_floyd_warshall() {
        // Directed weighted graph: 0->1 (1), 1->2 (2), 0->2 (5), 2->3 (1).
        let mut a = Matrix::filled(4, 4, MinPlus::Inf);
        a.set(0, 1, MinPlus::Fin(1)).unwrap();
        a.set(1, 2, MinPlus::Fin(2)).unwrap();
        a.set(0, 2, MinPlus::Fin(5)).unwrap();
        a.set(2, 3, MinPlus::Fin(1)).unwrap();
        let star = a.closure(AlgebraLimits::default()).unwrap();
        // shortest 0->2 is via 1: 1+2=3, beating the direct edge 5.
        assert_eq!(star.get(0, 2).unwrap(), &MinPlus::Fin(3));
        // 0->3 = 1+2+1 = 4.
        assert_eq!(star.get(0, 3).unwrap(), &MinPlus::Fin(4));
        // diagonal is 0 (empty path); unreachable stays Inf.
        assert_eq!(star.get(0, 0).unwrap(), &MinPlus::Fin(0));
        assert_eq!(star.get(3, 0).unwrap(), &MinPlus::Inf);
    }

    #[test]
    fn closure_without_star_reports_no_star() {
        use crate::real::RealF64;
        let a: Matrix<RealF64> = Matrix::filled(2, 2, RealF64(0.5));
        assert_eq!(
            a.closure(AlgebraLimits::default()).unwrap_err(),
            AlgebraError::NoStar
        );
    }
}
