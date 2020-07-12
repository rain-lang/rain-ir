/*!
Variable-width bitset types.
*/
use crate::eval::Apply;
use crate::typing::{Type, Typed};
use crate::value::{
    universe::FINITE_TY, NormalValue, TypeRef, UniverseRef, ValId, Value, ValueEnum,
};
use crate::{debug_from_display, enum_convert, quick_pretty, trivial_lifetime, trivial_substitute};

/// An n-bit number.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Bits(pub u16);

debug_from_display!(Bits);
quick_pretty!(Bits, s, fmt => write!(fmt, "#bits({})", s.0));
trivial_lifetime!(Bits);
trivial_substitute!(Bits);

enum_convert! {
    impl InjectionRef<ValueEnum> for Bits {}
    impl TryFrom<NormalValue> for Bits { as ValueEnum, }
    impl TryFromRef<NormalValue> for Bits { as ValueEnum, }
}

impl Apply for Bits {}

impl From<Bits> for NormalValue {
    fn from(bits: Bits) -> NormalValue {
        NormalValue(ValueEnum::Bits(bits))
    }
}

impl Type for Bits {
    #[inline]
    fn universe(&self) -> UniverseRef {
        FINITE_TY.borrow_var()
    }
    #[inline]
    fn is_universe(&self) -> bool {
        false
    }
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Typed for Bits {
    #[inline]
    fn ty(&self) -> TypeRef {
        FINITE_TY.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Value for Bits {
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
        ValueEnum::Bits(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}
