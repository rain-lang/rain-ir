/*!
Naive depth-first search, relying on a filter to reject already-seen elements
*/

use super::*;
use crate::value::{Value, ValId};

/// A naive depth-first search of a value's dependencies matching a given filter.
/// A depth-first search of a value's dependencies matching a given filter.
/// This filter maps the results, and may morph their dependencies and/or assert a certain value type.
/// Dependencies not matching the filter are ignored *along with all their descendants*.
/// This search relies on the filter to mark nodes as already visited: if not, expect an explosion of memory use.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NaiveDFS<V, F> {
    /// The frontier of this search
    frontier: Vec<V>,
    /// The filter to apply
    filter: F,
    /// The value type
    value: std::marker::PhantomData<V>,
}

impl<V, F> Iterator for NaiveDFS<V, F>
where
    V: Value,
    F: FnMut(&ValId) -> Option<V>,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        unimplemented!()
    }
}