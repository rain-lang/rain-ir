/*!
Reference-counted, hash-consed, typed arrays of values
*/

use super::{ValId, Value, VarId};
use crate::util::hash_cache::{Cache, Caches};
use itertools::Itertools;
use lazy_static::lazy_static;
use ref_cast::RefCast;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::{Deref, Index};
use triomphe::{Arc, HeaderSlice, HeaderWithLength, ThinArc};

lazy_static! {
    /// A cache for arrays of values
    pub static ref ARRAY_CACHE: Cache<ValId, ArcValIdArr> = Cache::default();
}

/// A reference-counted, hash-consed, typed array of values
#[derive(Debug, Clone, Eq, PartialEq, Hash, RefCast)]
#[repr(transparent)]
pub struct VarArr<V> {
    arr: PrivateValArr,
    variant: std::marker::PhantomData<V>,
}

impl<V> VarArr<V> {
    /// Get the length of this array
    #[inline]
    pub fn len(&self) -> usize {
        self.arr.len()
    }
    /// Check whether this array is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.arr.is_empty()
    }
    /// Get this array as an array of ValIds
    #[inline]
    pub fn as_vals(&self) -> &VarArr<ValIdMarker> {
        RefCast::ref_cast(&self.arr)
    }
    /// Check if this array is address-sorted
    #[inline]
    pub fn is_sorted(&self) -> bool {
        is_sorted::IsSorted::is_sorted_by_key(&mut self.arr.iter(), |v| v.as_ptr())
    }
    /// If this array is address-sorted, return it as a sorted array. If not, fail.
    #[inline]
    pub fn try_sorted(&self) -> Result<&VarSet<V>, ()> {
        if self.is_sorted() {
            Ok(RefCast::ref_cast(&self.arr))
        } else {
            Err(())
        }
    }
    /// Clone this array as an array of ValIds
    #[inline]
    pub fn clone_vals(&self) -> VarArr<ValIdMarker> {
        VarArr {
            arr: self.arr.clone(),
            variant: std::marker::PhantomData,
        }
    }
    /// Create a `VarArr` from an exact size iterator over `ValId`s, asserting it is of the desired type
    #[inline]
    fn assert_new<I: Iterator<Item = ValId> + ExactSizeIterator>(vals: I) -> VarArr<V> {
        Self::dedup_and_assert(ArcValIdArr(Arc::from_header_and_iter(
            HeaderWithLength::new((), vals.len()),
            vals,
        )))
    }
    /// Deduplicate an `Arc` to an array of `ValId`s, and assert this array is of the desired type
    #[inline]
    fn dedup_and_assert(ava: ArcValIdArr) -> VarArr<V> {
        let dedup_ava = ARRAY_CACHE.cache(ava);
        VarArr {
            arr: PrivateValArr(Arc::into_thin(dedup_ava.0)),
            variant: std::marker::PhantomData,
        }
    }
}

/// A marker for an array of ValIds
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ValIdMarker;

/// A marker for a *sorted* array of a given value type
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Sorted<T>(pub std::marker::PhantomData<T>);

impl<T> Sorted<T> {
    /// Create a new sorted array marker
    #[inline]
    pub fn new() -> Sorted<T> {
        Sorted(std::marker::PhantomData)
    }
}

/// An array of ValIds
pub type ValArr = VarArr<ValIdMarker>;

/// A set (implemented as a sorted array) of `rain` values
pub type VarSet<V> = VarArr<Sorted<V>>;

/// A set (implemented as a sorted array) of ValIds
pub type ValSet = VarArr<Sorted<ValIdMarker>>;

impl ValArr {
    /// Create a `ValArr` from an exact size iterator over `ValId`s
    #[inline]
    pub fn new<I: Iterator<Item = ValId> + ExactSizeIterator>(vals: I) -> ValArr {
        Self::assert_new(vals)
    }
    /// Deduplicate an `Arc` to an array of `ValId`s to get a `ValArr`
    #[inline]
    pub fn dedup(ava: ArcValIdArr) -> ValArr {
        Self::dedup_and_assert(ava)
    }
}

impl FromIterator<ValId> for ValArr {
    fn from_iter<I: IntoIterator<Item = ValId>>(iter: I) -> ValArr {
        let v: Vec<_> = iter.into_iter().collect();
        //TODO: optimize the case where size is known?
        Self::new(v.into_iter())
    }
}

impl FromIterator<ValId> for ValSet {
    fn from_iter<I: IntoIterator<Item = ValId>>(iter: I) -> ValSet {
        let v = iter.into_iter().sorted_by_key(ValId::as_ptr);
        Self::assert_new(v)
    }
}

impl<V: Value> FromIterator<V> for VarArr<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> VarArr<V> {
        let v: Vec<ValId> = iter.into_iter().map(Into::into).collect();
        Self::assert_new(v.into_iter())
    }
}

impl<V: Value> FromIterator<V> for VarSet<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> VarSet<V> {
        let v = iter
            .into_iter()
            .map(|v| v.into())
            .sorted_by_key(ValId::as_ptr);
        Self::assert_new(v)
    }
}

impl<V: Value> Index<usize> for VarArr<V> {
    type Output = VarId<V>;
    fn index(&self, ix: usize) -> &VarId<V> {
        RefCast::ref_cast(&self.arr[ix].0)
    }
}

impl Index<usize> for VarArr<ValIdMarker> {
    type Output = ValId;
    fn index(&self, ix: usize) -> &ValId {
        &self.arr[ix]
    }
}

impl Deref for VarArr<ValIdMarker> {
    type Target = [ValId];
    fn deref(&self) -> &[ValId] {
        &self.arr
    }
}

/// A reference-counted, hash-consed, typed array of values.
///
/// Implementation detail: Should not be constructable by the user!
#[derive(Clone, Eq, PartialEq)]
pub struct PrivateValArr(ThinArc<(), ValId>);

impl Deref for PrivateValArr {
    type Target = [ValId];
    fn deref(&self) -> &[ValId] {
        &self.0.slice
    }
}

impl Debug for PrivateValArr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.deref(), fmt)
    }
}

impl Hash for PrivateValArr {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

/// A reference-counted array of values which is not necessarily hash-consed.
/// This wrapper is for the `Hash` implementation
#[derive(Clone, Eq, PartialEq)]
pub struct ArcValIdArr(pub Arc<HeaderSlice<HeaderWithLength<()>, [ValId]>>);

impl Caches<ValId> for ArcValIdArr {
    #[inline]
    fn can_collect(&self) -> bool {
        self.0.is_unique()
    }
}

impl Deref for ArcValIdArr {
    type Target = [ValId];
    fn deref(&self) -> &[ValId] {
        &self.0.slice
    }
}

impl Debug for ArcValIdArr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.deref(), fmt)
    }
}

impl Hash for ArcValIdArr {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.deref().hash(hasher)
    }
}
