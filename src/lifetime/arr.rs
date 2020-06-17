/*!
`rain` lifetime arrays and sets
*/

use super::Lifetime;
use crate::util::cache::{
    arr::{CachedArr, EmptyPredicate, Sorted, Uniq},
    Cache,
};
use lazy_static::lazy_static;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

lazy_static! {
    /// The global lifetime array cache
    pub static ref LIFETIME_ARRAY_CACHE: Cache<[Lifetime], CachedArr<Lifetime>> = Cache::default();
}

/// An array of `rain` lifetimes optionally satisfying a given predicate `P`
#[derive(Debug, Clone, Eq)]
pub struct LifetimeArr<P = ()>(CachedArr<Lifetime, P>);

impl<P> LifetimeArr<P> {
    /// Deduplicate a `CachedArr` into a `LifetimeArr`
    pub fn dedup(cached: CachedArr<Lifetime, P>) -> LifetimeArr<P> {
        if cached.is_empty() {
            return LifetimeArr(cached);
        }
        LifetimeArr(LIFETIME_ARRAY_CACHE.cache(cached.coerce()).coerce())
    }
}

impl<P: EmptyPredicate> LifetimeArr<P> {
    /// An empty lifetime array
    pub const EMPTY: LifetimeArr<P> = LifetimeArr(CachedArr::EMPTY);
}

impl<P: EmptyPredicate> Default for LifetimeArr<P> {
    #[inline]
    fn default() -> LifetimeArr<P> {
        LifetimeArr::EMPTY
    }
}

impl<P, Q> PartialEq<LifetimeArr<P>> for LifetimeArr<Q> {
    #[inline]
    fn eq(&self, other: &LifetimeArr<P>) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl<P> Hash for LifetimeArr<P> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.as_ptr(), hasher)
    }
}

impl<P> Deref for LifetimeArr<P> {
    type Target = CachedArr<Lifetime, P>;
    #[inline]
    fn deref(&self) -> &CachedArr<Lifetime, P> {
        &self.0
    }
}

/// A bag of `rain` lifetimes
pub type LifetimeBag = LifetimeArr<Sorted>;

/// A set of `rain` lifetimes
pub type LifetimeSet = LifetimeArr<Uniq>;
