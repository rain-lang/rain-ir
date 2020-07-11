/*!
Graph-theoretic utilities for `rain`
*/
use crate::region::Regional;
use crate::value::{Deps, ValId, Value};
use fxhash::FxHashSet;
use smallvec::SmallVec;
use std::ops::RangeBounds;

pub mod dfs;

/// Filter already-visited addresses
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VisitedFilter(pub FxHashSet<usize>);

impl VisitedFilter {
    /// Create a new, empty visited filter
    pub fn new() -> VisitedFilter {
        VisitedFilter(FxHashSet::default())
    }
    /// Filter `ValId`'s, ignoring ones which have already been seen. Add seen ValIds to the seen set
    pub fn into_valid_filter(mut self) -> impl FnMut(&ValId) -> Option<&ValId> {
        move |valid| {
            let addr = valid.as_ptr() as usize;
            if !self.0.insert(addr) {
                return None;
            }
            Some(valid)
        }
    }
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
    /// Add this filter to an `FnMut`, consuming it
    pub fn into_filter<F, V>(mut self, mut filter: F) -> impl FnMut(&ValId) -> Option<V>
    where
        F: FnMut(&ValId) -> Option<V>,
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

const DEP_SEARCH_STACK_SIZE: usize = 16;

impl<V: Value> Deps<V> {
    /// Collect the immediate dependencies of this value within a given depth range which match a given filter
    pub fn collect_deps<R, F>(&self, range: R, filter: F) -> Vec<ValId>
    where
        V: Clone,
        R: RangeBounds<usize>,
        F: Fn(&ValId) -> bool,
    {
        let mut result = Vec::new();
        // Simple edge case
        if range.contains(&self.0.depth()) {
            return vec![self.0.clone().into_val()];
        }
        let mut searched = FxHashSet::<&ValId>::default();
        let mut frontier: SmallVec<[&ValId; DEP_SEARCH_STACK_SIZE]> = self.iter().collect();
        while let Some(dep) = frontier.pop() {
            searched.insert(dep);
            if range.contains(&dep.depth()) {
                if filter(dep) {
                    result.push(dep.clone())
                }
            } else {
                frontier.extend(dep.deps().iter().filter(|dep| !searched.contains(dep)))
            }
        }
        result
    }
}
