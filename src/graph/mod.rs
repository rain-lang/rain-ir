/*!
Graph-theoretic utilities for `rain`
*/
use crate::value::{Deps, ValId, Value};
use fxhash::FxHashSet;

pub mod dfs;

/// Filter already-visited addresses
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct VisitedFilter(pub FxHashSet<usize>);

/// A search result
#[derive(Debug, Clone)]
pub struct SearchResult<F, R> {
    /// What to add to the search frontier, if anything
    pub frontier: Option<F>,
    /// What to yield from the iterator, if anything
    pub result: Option<R>
}

/// A filter for `ValId`s
pub trait ValIdFilter<'a, V> {
    /// Filter a `ValId`, determining whether it is included
    fn filter(&mut self, value: &'a ValId) -> Option<&'a V>;
}

impl<'a, F, V: 'a> ValIdFilter<'a, V> for F
where
    F: FnMut(&'a ValId) -> Option<&'a V>,
{
    #[inline]
    fn filter(&mut self, value: &'a ValId) -> Option<&'a V> {
        self(value)
    }
}

impl VisitedFilter {
    /// Create a new, empty visited filter
    pub fn new() -> VisitedFilter {
        VisitedFilter(FxHashSet::default())
    }
}

impl<'a> ValIdFilter<'a, ValId> for VisitedFilter {
    #[inline]
    fn filter(&mut self, value: &'a ValId) -> Option<&'a ValId> {
        let addr = value.as_ptr() as usize;
        if self.0.insert(addr) {
            Some(value)
        } else {
            None
        }
    }
}

impl<'a> ValIdFilter<'a, ValId> for &'_ mut VisitedFilter {
    #[inline]
    fn filter(&mut self, value: &'a ValId) -> Option<&'a ValId> {
        (**self).filter(value)
    }
}

impl<V: Value> Deps<V> {
    /// Search the dependencies of a value matching a given predicate
    pub fn search<'a, P>(&'a self, mut predicate: P) -> impl Iterator<Item = &'a ValId> + 'a
    where
        V: Clone,
        P: FnMut(&ValId) -> bool + 'a,
    {
        let mut visited = VisitedFilter::new();
        let dfs: dfs::DepDFS<'a, _, _> = dfs::DepDFS::new(
            self.iter().collect(),
            move |value: &'a ValId| -> Option<&'a ValId> {
                visited.filter(value).filter(|value| predicate(*value))
            },
        );
        dfs
    }
}
