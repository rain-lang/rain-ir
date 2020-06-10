/*!
Predicates on a `rain` value
*/
use super::{NormalValue, ValId};
use std::convert::TryInto;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};

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
        v.as_norm().try_into().ok().expect("This predicate has been asserted valid!")
    }
}
