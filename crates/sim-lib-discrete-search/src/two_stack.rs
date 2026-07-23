//! Two-stack frontier adapter.

/// Deterministic two-stack adapter for layer-by-layer backtracking.
///
/// The current stack is popped until empty. Values pushed into the next stack
/// become the next layer, preserving insertion order within that layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TwoStackAdapter<T> {
    current: Vec<T>,
    next: Vec<T>,
    depth: usize,
}

impl<T> TwoStackAdapter<T> {
    /// Build an adapter seeded with one root item.
    pub fn with_root(root: T) -> Self {
        Self {
            current: vec![root],
            next: Vec::new(),
            depth: 0,
        }
    }

    /// Push an item into the next layer.
    pub fn push_next(&mut self, item: T) {
        self.next.push(item);
    }

    /// Pop the next item, advancing a layer when the current stack is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.current.is_empty() && !self.next.is_empty() {
            self.next.reverse();
            std::mem::swap(&mut self.current, &mut self.next);
            self.depth += 1;
        }
        self.current.pop()
    }

    /// Current layer depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Number of queued items across both stacks.
    pub fn len(&self) -> usize {
        self.current.len() + self.next.len()
    }

    /// Return true when no item is queued in either stack.
    pub fn is_empty(&self) -> bool {
        self.current.is_empty() && self.next.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_preserves_next_layer_insertion_order() {
        let mut frontier = TwoStackAdapter::with_root("root");
        assert_eq!(frontier.pop(), Some("root"));
        frontier.push_next("a");
        frontier.push_next("b");
        assert_eq!(frontier.depth(), 0);
        assert_eq!(frontier.pop(), Some("a"));
        assert_eq!(frontier.depth(), 1);
        assert_eq!(frontier.pop(), Some("b"));
        assert!(frontier.is_empty());
    }
}
