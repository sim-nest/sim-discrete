//! Integer partitions of `n` in ascending-composition order (Kelleher's
//! `accel_asc`). The iterator is genuinely lazy, so large `n` is safe to start.

/// Iterator over the partitions of `n`, each as an ascending list of parts.
#[derive(Debug, Clone)]
pub struct IntegerPartitionIter {
    a: Vec<usize>,
    k: usize,
    done: bool,
    emit_empty: bool,
}

/// Construct a partition iterator for `n`. `n == 0` yields the empty partition.
pub fn integer_partitions(n: usize) -> IntegerPartitionIter {
    if n == 0 {
        return IntegerPartitionIter {
            a: Vec::new(),
            k: 0,
            done: true,
            emit_empty: true,
        };
    }
    let mut a = vec![0usize; n + 1];
    a[1] = n;
    IntegerPartitionIter {
        a,
        k: 1,
        done: false,
        emit_empty: false,
    }
}

impl Iterator for IntegerPartitionIter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.emit_empty {
            self.emit_empty = false;
            return Some(Vec::new());
        }
        if self.done || self.k == 0 {
            return None;
        }
        let mut k = self.k;
        let x = self.a[k - 1] + 1;
        let mut y = self.a[k] - 1;
        k -= 1;
        while x <= y {
            self.a[k] = x;
            y -= x;
            k += 1;
        }
        self.a[k] = x + y;
        let result = self.a[0..=k].to_vec();
        self.k = k;
        if k == 0 {
            self.done = true;
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::count::integer_partition_count;
    use num_bigint::BigUint;

    #[test]
    fn count_matches_closed_form() {
        let parts: Vec<_> = integer_partitions(5).collect();
        assert_eq!(parts.len(), 7);
        assert_eq!(
            BigUint::from(parts.len() as u32),
            integer_partition_count(5)
        );
        // Each partition sums to n.
        for p in &parts {
            assert_eq!(p.iter().sum::<usize>(), 5);
        }
    }

    #[test]
    fn first_and_last() {
        let parts: Vec<_> = integer_partitions(5).collect();
        assert_eq!(parts[0], vec![1, 1, 1, 1, 1]);
        assert_eq!(parts[6], vec![5]);
    }

    #[test]
    fn zero_yields_empty_partition() {
        let parts: Vec<_> = integer_partitions(0).collect();
        assert_eq!(parts, vec![Vec::<usize>::new()]);
    }
}
