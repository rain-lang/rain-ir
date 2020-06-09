/*!
Reference-counted, hash-consed, typed arrays of values
*/

use super::{ValId, Value, VarId};
use ref_cast::RefCast;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, Index};
use triomphe::ThinArc;

/// A reference-counted, hash-consed, typed array of values
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
}

/// A marker for a ValId
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ValIdMarker;

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
