//! Disjoint-set union-find with path compression and union by rank. Used by
//! Kruskal's MST and by the MST verifier (acyclicity / connectivity checks).

/// A disjoint-set forest over `0..n`.
#[derive(Debug, Clone)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    /// `n` singleton sets `{0}, {1}, ..., {n-1}`.
    pub fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    /// The representative of `x`'s set, with path compression.
    pub fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut cur = x;
        while self.parent[cur] != root {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        root
    }

    /// Union the sets of `a` and `b`. Returns `true` if they were distinct (a
    /// merge happened), `false` if they were already joined (would form a cycle).
    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => self.parent[ra] = rb,
            std::cmp::Ordering::Greater => self.parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.rank[ra] += 1;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_reports_cycle() {
        let mut uf = UnionFind::new(3);
        assert!(uf.union(0, 1));
        assert!(uf.union(1, 2));
        // 0 and 2 are already connected; unioning again would be a cycle.
        assert!(!uf.union(0, 2));
        assert_eq!(uf.find(0), uf.find(2));
    }
}
