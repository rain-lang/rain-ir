/*!
Finite-valued types
*/
use crate::eval::Apply;
use crate::region::Regional;
use crate::tokens::*;
use crate::typing::{primitive::FIN, Type, Typed};
use crate::value::{NormalValue, TypeRef, ValId, Value, ValueData, ValueEnum, VarId, VarRef};
use crate::{debug_from_display, enum_convert, quick_pretty, trivial_substitute};
use num::ToPrimitive;
use ref_cast::RefCast;
use std::cmp::Ordering;
use std::ops::Deref;

/// A type with `n` values
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, RefCast)]
#[repr(transparent)]
pub struct BitsTy(pub u64);

debug_from_display!(BitsTy);
quick_pretty!(BitsTy, "Unimplemented!");
trivial_substitute!(BitsTy);
enum_convert! {
    impl InjectionRef<ValueEnum> for BitsTy {}
    impl TryFrom<NormalValue> for BitsTy { as ValueEnum, }
    impl TryFromRef<NormalValue> for BitsTy { as ValueEnum, }
}

impl Typed for BitsTy {
    #[inline]
    fn ty(&self) -> TypeRef {
        FIN.borrow_ty()
    }

    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Apply for BitsTy {}

impl Regional for BitsTy {}

impl Value for BitsTy {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Tried to get dependency #{} of bits type {}, which has none",
            ix, self
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::BitsTy(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl From<BitsTy> for NormalValue {
    fn from(b: BitsTy) -> NormalValue {
        b.into_norm()
    }
}