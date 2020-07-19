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
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct VisitedFilter(pub FxHashSet<usize>);

/// A filter for `ValId`s
pub trait ValIdFilter<V> {
    /// Filter a `ValId`, determining whether it is included
    fn filter<'a>(&mut self, value: &'a ValId) -> Option<&'a V>;
}

impl<F, V> ValIdFilter<V> for F where F: FnMut(&ValId) -> Option<&V> {
    #[inline]
    fn filter<'a>(&mut self, value: &'a ValId) -> Option<&'a V> {
        self(value)
    }
}

impl VisitedFilter {
    /// Create a new, empty visited filter
    pub fn new() -> VisitedFilter {
        VisitedFilter(FxHashSet::default())
    }
}

impl ValIdFilter<ValId> for VisitedFilter {
    #[inline]
    fn filter<'a>(&mut self, value: &'a ValId) -> Option<&'a ValId> {
        let addr = value.as_ptr() as usize;
        if self.0.insert(addr) {
            Some(value)
        } else {
            None
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
