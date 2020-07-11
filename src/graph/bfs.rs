/*!
Breadth-first search
*/

use super::*;
use crate::value::{Value, ValId};

/// A breadth-first search of a value's dependencies matching a given filter.
/// Dependencies not matching the filter are ignored *along with all their descendants*.
/// May repeat dependencies.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DepBFS<V, F> {
    /// The frontier of this search
    frontier: Vec<V>,
    /// The filter to apply
    filter: F,
    /// The value type
    value: std::marker::PhantomData<V>,
}

impl<V, F> Iterator for DepBFS<V, F>
where
    V: Value,
    F: FnMut(&ValId) -> Option<V>,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        unimplemented!()
    }
}
