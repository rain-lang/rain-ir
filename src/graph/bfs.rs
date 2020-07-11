/*!
Breadth-first search
*/

use crate::value::{ValId, Value};

/// A breadth-first search of a value's dependencies matching a given filter.
/// Dependencies not matching the filter are ignored *along with all their descendants*.
/// Fallible filters are supported: in this case, the search will halt.
/// May repeat dependencies.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DepBFS<V, F> {
    /// The frontier of this search
    frontier: Vec<V>,
    /// The filter to apply
    filter: F,
}

impl<V, F> DepBFS<V, F> {
    /// Create a new BFS starting at a given frontier
    #[inline]
    pub fn new(frontier: Vec<V>, filter: F) -> DepBFS<V, F> {
        DepBFS { frontier, filter }
    }
    /// Create a new BFS starting at a given value
    #[inline]
    pub fn new_at(start: V, filter: F) -> DepBFS<V, F> {
        Self::new(vec![start], filter)
    }
}

impl<V, F> Iterator for DepBFS<V, F>
where
    V: Value,
    F: FnMut(&ValId) -> Option<V>,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        let top = self.frontier.pop()?;
        self.frontier
            .extend(top.deps().iter().filter_map(&mut self.filter));
        Some(top)
    }
}
