/*!
Reference-counted, hash-consed, typed arrays of values
*/

use super::{ValId, Value, VarId};
use crate::util::hash_cache::{Cache, Caches};
use lazy_static::lazy_static;
use ref_cast::RefCast;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
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
    /// Clone this array as an array of ValIds
    #[inline]
    pub fn clone_vals(&self) -> VarArr<ValIdMarker> {
        VarArr {
            arr: self.arr.clone(),
            variant: std::marker::PhantomData,
        }
    }
}

/// A marker for an array of ValIds
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ValIdMarker;

/// An array of ValIds
pub type ValArr = VarArr<ValIdMarker>;

impl ValArr {
    /// Create a `ValArr` from (exact size) iterator over `ValId`s
    pub fn new<I: Iterator<Item = ValId> + ExactSizeIterator>(vals: I) -> ValArr {
        Self::dedup(ArcValIdArr(Arc::from_header_and_iter(
            HeaderWithLength::new((), vals.len()),
            vals,
        )))
    }
    /// Deduplicate an `Arc` to an array of `ValId`s to get a `ValArr`
    pub fn dedup(ava: ArcValIdArr) -> ValArr {
        let dedup_ava = ARRAY_CACHE.cache(ava);
        ValArr {
            arr: PrivateValArr(Arc::into_thin(dedup_ava.0)),
            variant: std::marker::PhantomData,
        }
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
