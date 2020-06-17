/*!
Cached immutable arrays, bags, and sets of values
*/

use super::Caches;
use itertools::Itertools;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;
use triomphe::{Arc, HeaderWithLength, ThinArc};

/// A cached array satisfying a given predicate `P`
#[repr(transparent)]
pub struct CachedArr<A, P = (), H = ()> {
    ptr: Option<ThinArc<H, A>>,
    predicate: std::marker::PhantomData<P>,
}

impl<A, P: EmptyPredicate> Default for CachedArr<A, P> {
    fn default() -> CachedArr<A, P> {
        CachedArr::EMPTY
    }
}

impl<A: Eq + Deref, P> Caches<[A]> for CachedArr<A, P> {
    #[inline]
    fn can_collect(&self) -> bool {
        if let Some(ptr) = &self.ptr {
            ptr.with_arc(|ptr| ptr.is_unique())
        } else {
            true
        }
    }
}

impl<A, P, H> Clone for CachedArr<A, P, H> {
    #[inline]
    fn clone(&self) -> CachedArr<A, P, H> {
        CachedArr {
            ptr: self.ptr.clone(),
            predicate: self.predicate,
        }
    }
}

impl<A: Deref, P, Q, H> PartialEq<CachedArr<A, P, H>> for CachedArr<A, Q, H> {
    #[inline]
    fn eq(&self, other: &CachedArr<A, P, H>) -> bool {
        let lhs = self.as_slice();
        let rhs = other.as_slice();
        if lhs.len() != rhs.len() {
            return false;
        }
        for (l, r) in lhs.iter().zip(rhs.iter()) {
            if l.deref() as *const A::Target != r.deref() as *const A::Target {
                return false;
            }
        }
        return true;
    }
}

impl<A: Deref, P, H> Eq for CachedArr<A, P, H> {}

impl<A, P, H> CachedArr<A, P, H> {
    /// Get the pointer to the first element of this `CachedArr`, or null if there is none (i.e. the slice is empty)
    #[inline]
    pub fn as_ptr(&self) -> *const A {
        self.as_slice()
            .first()
            .map(|f| f as *const A)
            .unwrap_or(std::ptr::null())
    }
    /// Get the slice underlying this `CachedArr`
    #[inline]
    pub fn as_slice(&self) -> &[A] {
        self.ptr.as_ref().map(|p| &p.slice).unwrap_or(&[])
    }
    /// Iterate over the items of this `CachedArr`
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<A> {
        self.as_slice().iter()
    }
    /// Strip the predicate from this `CachedArr`
    #[inline]
    pub fn as_arr(&self) -> &CachedArr<A, (), H> {
        self.coerce_ref()
    }
    /// Strip the predicate from this `CachedArr`, consuming it.
    #[inline]
    pub fn into_arr(self) -> CachedArr<A, (), H> {
        self.coerce()
    }
    /// Coerce this array into one satisfying another predicate
    #[inline]
    pub fn coerce<Q>(self) -> CachedArr<A, Q, H> {
        CachedArr {
            ptr: self.ptr,
            predicate: std::marker::PhantomData,
        }
    }
    /// Coerce this array as a reference into one satisfying another predicate
    #[inline]
    pub fn coerce_ref<Q>(&self) -> &CachedArr<A, Q, H> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<A, P> Debug for CachedArr<A, P>
where
    A: Debug,
{
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.as_slice(), fmt)
    }
}

impl<A, P> Hash for CachedArr<A, P>
where
    A: Deref,
{
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        for value in self.as_slice().iter() {
            std::ptr::hash(value.deref() as *const _, hasher)
        }
    }
}

impl<A, P> Deref for CachedArr<A, P> {
    type Target = [A];
    #[inline]
    fn deref(&self) -> &[A] {
        self.as_slice()
    }
}

impl<A> CachedArr<A> {
    /// Create a new cached array from an exact length iterator
    pub fn from_exact<I: ExactSizeIterator + Iterator<Item = A>>(iter: I) -> CachedArr<A> {
        if iter.len() == 0 {
            CachedArr::EMPTY
        } else {
            let ptr = Arc::from_header_and_iter(HeaderWithLength::new((), iter.len()), iter);
            CachedArr {
                ptr: Some(Arc::into_thin(ptr)),
                predicate: std::marker::PhantomData,
            }
        }
    }
}

impl<A> From<Vec<A>> for CachedArr<A> {
    fn from(v: Vec<A>) -> CachedArr<A> {
        Self::from_exact(v.into_iter())
    }
}

impl<A: Clone> From<&'_ [A]> for CachedArr<A> {
    fn from(v: &[A]) -> CachedArr<A> {
        Self::from_exact(v.iter().cloned())
    }
}

impl<A: Deref> From<Vec<A>> for CachedBag<A> {
    fn from(mut v: Vec<A>) -> CachedBag<A> {
        v.sort_unstable_by_key(|a| a.deref() as *const _);
        CachedArr::<A>::from(v).coerce()
    }
}

impl<A: Deref> From<Vec<A>> for CachedSet<A> {
    fn from(mut v: Vec<A>) -> CachedSet<A> {
        v.sort_unstable_by_key(|a| (*a).deref() as *const _);
        v.dedup_by_key(|a| (*a).deref() as *const _);
        CachedArr::<A>::from(v).coerce()
    }
}

impl<A> FromIterator<A> for CachedArr<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> CachedArr<A> {
        iter.into_iter().collect_vec().into()
    }
}

impl<A: Deref> FromIterator<A> for CachedBag<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> CachedBag<A> {
        iter.into_iter().collect_vec().into()
    }
}

impl<A: Deref> FromIterator<A> for CachedSet<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> CachedSet<A> {
        iter.into_iter().collect_vec().into()
    }
}

/// A marker type for a predicate which is true for any empty array
pub trait EmptyPredicate {}

impl EmptyPredicate for () {}

impl<A, P: EmptyPredicate> CachedArr<A, P> {
    /// Get a constant empty `CachedArr`
    pub const EMPTY: CachedArr<A, P> = CachedArr {
        ptr: None,
        predicate: std::marker::PhantomData,
    };
}

/// A marker type indicating an array which is sorted by address, but may have duplicates
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Sorted;

/// A marker trait indicating that `CachedArr`s satisfying this predicate may be used as a bag
pub trait BagMarker {}

impl BagMarker for Sorted {}
impl EmptyPredicate for Sorted {}

/// A marker type indicating an array which is strictly sorted by address (i.e. no duplicates)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Uniq;

/// A marker trait indicating that `CachedArr`s satisfying this predicate may be used as a set
pub trait SetMarker: BagMarker {}

impl BagMarker for Uniq {}
impl SetMarker for Uniq {}
impl EmptyPredicate for Uniq {}

impl<A: Deref, P, H> CachedArr<A, P, H> {
    /// Check if this array is sorted by address
    pub fn is_sorted(&self) -> bool {
        self.as_slice()
            .windows(2)
            .all(|w| w[0].deref() as *const _ <= w[1].deref() as *const _)
    }
    /// Check if this array is strictly sorted by address
    pub fn is_set(&self) -> bool {
        self.as_slice()
            .windows(2)
            .all(|w| w[0].deref() as *const _ < w[1].deref() as *const _)
    }
    /// Try to cast this array into a bag if sorted
    pub fn try_as_bag(&self) -> Result<&CachedBag<A, H>, &Self> {
        if self.is_sorted() {
            Ok(self.coerce_ref())
        } else {
            Err(self)
        }
    }
    /// Try to cast this array into a set if strictly sorted
    pub fn try_as_set(&self) -> Result<&CachedSet<A, H>, &Self> {
        if self.is_set() {
            Ok(self.coerce_ref())
        } else {
            Err(self)
        }
    }
    /// Sort this array and return it
    pub fn sorted(&self) -> CachedBag<A>
    where
        A: Clone,
    {
        CachedBag::from(self.iter().cloned().collect_vec())
    }
    /// Sort and deduplicate this array and return it
    pub fn set(&self) -> CachedSet<A>
    where
        A: Clone,
    {
        CachedSet::from(self.iter().cloned().collect_vec())
    }
    /// Try to cast this array into a bag if sorted
    pub fn try_into_bag(self) -> Result<CachedBag<A, H>, Self> {
        if self.is_sorted() {
            Ok(self.coerce())
        } else {
            Err(self)
        }
    }
    /// Try to cast this array into a set if strictly sorted
    pub fn try_into_set(self) -> Result<CachedSet<A, H>, Self> {
        if self.is_set() {
            Ok(self.coerce())
        } else {
            Err(self)
        }
    }
}

impl<A: Deref, P: BagMarker, H> CachedArr<A, P, H> {
    /// Cast this array into a bag
    pub fn as_bag(&self) -> &CachedBag<A, H> {
        self.coerce_ref()
    }
    /// Cast this array into a bag
    pub fn into_bag(self) -> CachedBag<A, H> {
        self.coerce()
    }
}

impl<A: Deref, P: SetMarker, H> CachedArr<A, P, H> {
    /// Cast this array into a set
    pub fn as_set(&self) -> &CachedSet<A, H> {
        self.coerce_ref()
    }
    /// Cast this array into a set
    pub fn into_set(self) -> CachedSet<A, H> {
        self.coerce()
    }
}

/// A cached bag of elements
pub type CachedBag<A, H = ()> = CachedArr<A, Sorted, H>;

impl<A: Deref + Clone, H> CachedBag<A, H> {
    /// Check whether an item is in this bag. If it is, return a reference.
    pub fn contains_impl(&self, item: *const A::Target) -> Option<&A> {
        self.as_slice()
            .binary_search_by_key(&item, |a| (*a).deref() as *const A::Target)
            .ok()
            .map(|ix| &self.as_slice()[ix])
    }
    /// Deduplicate this bag into a set
    pub fn uniq_impl(&self) -> CachedSet<A> {
        let mut v = self.iter().cloned().collect_vec();
        v.dedup_by_key(|a| (*a).deref() as *const A::Target);
        CachedArr::<A>::from(v).coerce()
    }
}

impl<A: Deref + Clone> CachedBag<A> {
    /// Merge two bags
    pub fn merge_impl(&self, rhs: &CachedBag<A>) -> CachedBag<A> {
        // Edge cases
        if rhs.is_empty() {
            return self.clone();
        } else if self.is_empty() {
            return rhs.clone();
        }
        let union = self
            .iter()
            .merge_by(rhs.iter(), |l, r| {
                (*l).deref() as *const A::Target <= (*r).deref() as *const A::Target
            })
            .cloned()
            .collect_vec();
        CachedArr::<A>::from(union).coerce()
    }
    /// Take the intersection of two bags
    pub fn intersect_impl(&self, rhs: &CachedBag<A>) -> CachedBag<A> {
        // Edge cases
        if rhs.is_empty() {
            return rhs.clone();
        } else if self.is_empty() {
            return self.clone();
        }
        let intersection = self
            .iter()
            .merge_join_by(rhs.iter(), |l, r| {
                ((*l).deref() as *const A::Target).cmp(&((*r).deref() as *const A::Target))
            })
            .filter_map(|v| v.both().map(|(l, _)| l))
            .cloned()
            .collect_vec();
        CachedArr::<A>::from(intersection).coerce()
    }
}

impl<A: Deref + Clone, P: BagMarker, H> CachedArr<A, P, H> {
    /// Check whether an item is in this bag. If it is, return a reference.
    pub fn contains(&self, item: *const A::Target) -> Option<&A> {
        self.coerce_ref().contains_impl(item)
    }
}

impl<A: Deref + Clone, P: BagMarker> CachedArr<A, P> {
    /// Merge two bags
    #[inline]
    pub fn merge<Q: BagMarker>(&self, rhs: &CachedArr<A, Q>) -> CachedBag<A> {
        self.coerce_ref().merge_impl(rhs.coerce_ref())
    }
    /// Take the intersection of two bags
    #[inline]
    pub fn intersect<Q: BagMarker>(&self, rhs: &CachedArr<A, Q>) -> CachedArr<A, P> {
        self.coerce_ref().intersect_impl(rhs.coerce_ref()).coerce()
    }
    /// Deduplicate this bag into a set
    #[inline]
    pub fn uniq(&self) -> CachedSet<A> {
        self.coerce_ref().uniq_impl()
    }
}

/// A cached set of elements
pub type CachedSet<A, H = ()> = CachedArr<A, Uniq, H>;

impl<A: Deref + Clone> CachedSet<A> {
    /// Take the union of two sets
    pub fn union_impl(&self, rhs: &CachedSet<A>) -> CachedSet<A> {
        // Edge cases
        if rhs.is_empty() {
            return self.clone();
        } else if self.is_empty() {
            return rhs.clone();
        }
        let union = self
            .iter()
            .merge_join_by(rhs.iter(), |l, r| {
                ((*l).deref() as *const A::Target).cmp(&((*r).deref() as *const A::Target))
            })
            .map(|v| v.reduce(|l, _| l))
            .cloned()
            .collect_vec();
        CachedArr::<A>::from(union).coerce()
    }
}

impl<A: Deref + Clone, P: SetMarker> CachedArr<A, P> {
    /// Take the union of two sets
    pub fn union<Q: SetMarker>(&self, rhs: &CachedArr<A, Q>) -> CachedSet<A> {
        self.coerce_ref().union_impl(rhs.coerce_ref())
    }
}
