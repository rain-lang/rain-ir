/*!
Cached immutable arrays, bags, and sets of values
*/

use itertools::Itertools;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;
use triomphe::{Arc, HeaderWithLength, ThinArc};

/// A cached array satisfying a given predicate `P`
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct CachedArr<A, P = (), H = ()> {
    ptr: Option<ThinArc<H, A>>,
    predicate: std::marker::PhantomData<P>,
}

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
    fn coerce<Q>(self) -> CachedArr<A, Q, H> {
        CachedArr {
            ptr: self.ptr,
            predicate: std::marker::PhantomData,
        }
    }
    /// Coerce this array as a reference into one satisfying another predicate
    #[inline]
    fn coerce_ref<Q>(&self) -> &CachedArr<A, Q, H> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<A> Debug for CachedArr<A>
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

impl<A> From<Vec<A>> for CachedArr<A> {
    fn from(v: Vec<A>) -> CachedArr<A> {
        if v.len() == 0 {
            CachedArr::empty()
        } else {
            let ptr = Arc::from_header_and_iter(HeaderWithLength::new((), v.len()), v.into_iter());
            CachedArr {
                ptr: Some(Arc::into_thin(ptr)),
                predicate: std::marker::PhantomData,
            }
        }
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
        v.sort_unstable_by_key(|a| a.deref() as *const _);
        v.dedup_by_key(|a| a.deref() as *const _);
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
    /// Create an empty cached array
    pub fn empty() -> CachedArr<A, P> {
        CachedArr {
            ptr: None,
            predicate: std::marker::PhantomData,
        }
    }
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
        is_sorted::IsSorted::is_sorted_by_key(&mut self.as_slice().iter(), |v| {
            v.deref() as *const _
        })
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

/// A cached set of elements
pub type CachedSet<A, H = ()> = CachedArr<A, Uniq, H>;
