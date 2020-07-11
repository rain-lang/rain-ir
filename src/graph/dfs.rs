/*!
Depth-first search
*/

use super::*;
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

impl<'a, V, F> Iterator for DepDFS<Borrowed<'a, V>, F>
where
    V: Value,
    F: FnMut(&'a ValId) -> Option<&'a V>,
{
    type Item = &'a V;
    fn next(&mut self) -> Option<&'a V> {
        loop {
            let mut push_to_top = None;
            {
                let (top, ix) = self.frontier.last_mut()?;
                while *ix < top.0.no_deps() {
                    *ix += 1;
                    if let Some(dep) = (self.filter)(top.0.get_dep(*ix - 1)) {
                        push_to_top = Some(Borrowed(dep));
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
        self.frontier.pop().map(|(b, _)| b.0)
    }
}
