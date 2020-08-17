/*!
Predicates on a `rain` value
*/
use super::{NormalValue, ValId, ValueEnum};
use std::convert::TryInto;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// A `NormalValue` satisfying a given predicate
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct PredicatedValue<P> {
    value: NormalValue,
    predicate: std::marker::PhantomData<P>,
}

impl<P> PredicatedValue<P> {
    /// Coerce a `ValueEnum` into a predicated value
    #[inline(always)]
    pub(crate) fn coerce_value(value: &ValueEnum) -> &PredicatedValue<P> {
        unsafe { &*(value as *const _ as *const PredicatedValue<P>) }
    }
    /// Coerce a `NormalValue` into a predicated value
    #[inline(always)]
    pub(crate) fn coerce_norm(value: &NormalValue) -> &PredicatedValue<P> {
        unsafe { &*(value as *const _ as *const PredicatedValue<P>) }
    }
}

impl<P> Deref for PredicatedValue<P> {
    type Target = NormalValue;
    #[inline]
    fn deref(&self) -> &NormalValue {
        &self.value
    }
}

/// A predicate indicating a `rain` value is of a certain type
pub struct Is<V>(pub std::marker::PhantomData<V>);

impl<V> Is<V> {
    /// The constant, default value for `Is<V>`
    pub const DEF: Is<V> = Is(std::marker::PhantomData);
}

impl<V> Debug for Is<V> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Is")
    }
}

impl<V> Clone for Is<V> {
    #[inline]
    fn clone(&self) -> Is<V> {
        Is::DEF
    }
}

impl<V> Copy for Is<V> {}

impl<V> Hash for Is<V> {
    #[inline]
    fn hash<H: Hasher>(&self, _hasher: &mut H) {}
}

impl<V> PartialEq for Is<V> {
    #[inline]
    fn eq(&self, _other: &Is<V>) -> bool {
        true
    }
}

impl<V> Eq for Is<V> {}

/// A predicate which allows borrowing a given value type
pub trait BorrowPredicate {
    /// The value type which can be borrowed
    type Borrows;
    /// Borrow the value type
    fn borrow_value(v: &ValId) -> &Self::Borrows;
}

impl<V> BorrowPredicate for Is<V>
where
    for<'a> &'a NormalValue: TryInto<&'a V>,
{
    type Borrows = V;
    #[inline]
    fn borrow_value(v: &ValId) -> &V {
        v.as_norm()
            .try_into()
            .ok()
            .expect("This predicate has been asserted valid!")
    }
}
