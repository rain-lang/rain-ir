/*!
Graph-theoretic utilities for `rain`
*/
use crate::value::ValId;
use fxhash::FxHashSet;

pub mod bfs;
pub mod dfs;

/// Filter already-visited addresses
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VisitedFilter(pub FxHashSet<usize>);

impl VisitedFilter {
    /// Filter `ValId`'s, ignoring ones which have already been seen. Add seen ValIds to the seen set
    pub fn valid_filter(&mut self) -> impl FnMut(&ValId) -> Option<&ValId> + '_ {
        move |valid| {
            let addr = valid.as_ptr() as usize;
            if !self.0.insert(addr) {
                return None;
            }
            Some(valid)
        }
    }
    /// Filter `ValId`s, ignoring ones which have already been seen. Do not modify the seen set
    pub fn static_valid_filter(&self) -> impl Fn(&ValId) -> Option<&ValId> + '_ {
        move |valid| {
            let addr = valid.as_ptr() as usize;
            if self.0.contains(&addr) {
                return None;
            }
            return Some(valid);
        }
    }
    /// Add this filter to an `FnMut`
    pub fn filter<'a, F, V>(&'a mut self, mut filter: F) -> impl FnMut(&ValId) -> Option<V> + 'a
    where
        F: FnMut(&ValId) -> Option<V> + 'a,
    {
        move |valid| {
            let addr = valid.as_ptr() as usize;
            if !self.0.insert(addr) {
                return None;
            }
            filter(valid)
        }
    }
}
