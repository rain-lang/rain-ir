/*!
A cache for hash-consing values
*/

use dashmap::{DashMap, RandomState};
use std::cell::RefCell;
use triomphe::Arc;
use std::hash::{Hash, BuildHasher};
use std::ops::Deref;

/// A cache for values of type `T`
#[derive(Debug, Default)]
pub struct Cache<T, S = RandomState> {
    /// The set of cached values
    cache: DashMap<Arc<T>, (), RandomState>,
}

impl<T: Hash> Cache<T> {
    /// Create a new, empty cache
    pub fn new() -> Cache {
        Cache { cache: DashSet::new() }
    }
}

impl<T: Hash, S: BuildHasher> Cache<T, S> {
    /// Attempt to cache a value. If already cached, return the corresponding `Arc`
    pub fn cache(&self, value: Q) -> Arc<T> where Arc<T>: Borrow<Q>, Q: Into<Arc<T>> {
        // Read-lock first!
        // TODO: profile and see if this actually even helps efficiency at all
        if let Some(cached) = self.cache.get(&value) { c
            return cached.clone()
        }
        self.cache.entry(value.into()).or_default().key().clone()
    }
}