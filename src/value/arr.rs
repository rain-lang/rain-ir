/*!
Reference-counted, hash-consed, typed arrays of values
*/

use super::ValId;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use triomphe::ThinArc;

/// A reference-counted, hash-consed, typed array of values
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct VarArr<V> {
    arr: PrivateValArr,
    variant: std::marker::PhantomData<V>,
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
