/*!
Depth-first search
*/

use crate::value::{Value, ValId};

/// A depth-first search of a value's dependencies matching a given filter.
/// This filter maps the results, and may morph their dependencies and/or assert a certain value type.
/// Dependencies not matching the filter are ignored *along with all their descendants*.
/// May repeat dependencies.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DepDFS<V, F> {
    /// The frontier of this search
    frontier: Vec<(V, usize)>,
    /// The filter to apply
    filter: F,
}

impl<V, F> DepDFS<V, F> {
    /// Create a new BFS starting at a given position frontier
    #[inline]
    pub fn new(frontier: Vec<(V, usize)>, filter: F) -> DepDFS<V, F> {
        DepDFS { frontier, filter }
    }
    /// Create a new BFS starting at a given value
    #[inline]
    pub fn new_at(start: V, filter: F) -> DepDFS<V, F> {
        Self::new(vec![(start, 0)], filter)
    }
}

impl<V, F> Iterator for DepDFS<V, F>
where
    V: Value,
    F: FnMut(&ValId) -> Option<V>,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        loop {
            let mut push_to_top = None;
            {
                let (top, ix) = self.frontier.last_mut()?;
                while *ix < top.no_deps() {
                    *ix += 1;
                    if let Some(dep) = (self.filter)(top.get_dep(*ix - 1)) {
                        push_to_top = Some(dep);
                        break; // Push this to the top of the dependency stack, repeat
                    }
                }
            }
            if let Some(to_push) = push_to_top {
                self.frontier.push((to_push, 0));
                continue;
            } else {
                break;
            }
        }
        self.frontier.pop().map(|(b, _)| b)
    }
}