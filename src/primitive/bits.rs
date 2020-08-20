/*!
Finite-valued types
*/
use crate::eval::Apply;
use crate::region::Regional;
use crate::typing::{primitive::FIN, Type, Typed};
use crate::value::{NormalValue, TypeRef, ValId, Value, ValueEnum, VarId, Error, VarRef, ValueData};
use crate::{debug_from_display, enum_convert, quick_pretty, trivial_substitute};
use num::ToPrimitive;
use ref_cast::RefCast;
use std::ops::Deref;
use std::convert::TryInto;

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

impl BitsTy {
    /// Get a bitset into this type. Return an error if too many bits
    pub fn data<I: ToPrimitive>(self, data: I) -> Result<Bits, Error> {
        let data = if let Some(data) = data.to_u128() {
            data
        } else {
            return Err(Error::TooManyBits);
        };
        Bits::try_new(self, data)
    }
}

impl ValueData for BitsTy {}

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
         NormalValue::assert_normal(ValueEnum::BitsTy(self))
    }
}

impl From<BitsTy> for NormalValue {
    fn from(b: BitsTy) -> NormalValue {
        b.into_norm()
    }
}

impl Type for BitsTy {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

/// A bitset
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Bits {
    /// The type this bitset is part of
    ty: VarId<BitsTy>,
    /// The data this bit set stores
    data: u128,
}

impl Bits {
    /// Try to construct a new bitset. Return an error if high bits are set.
    pub fn try_new<B: Into<VarId<BitsTy>>>(ty: B, data: u128) -> Result<Bits, Error> {
        let ty: VarId<BitsTy> = ty.into();
        let width: u32 = ty.deref().0.try_into().map_err(|_| Error::TooManyBits)?;
        if width > 128 || data.wrapping_shr(width.min(127)) != 0 {
            Err(Error::TooManyBits)
        } else {
            Ok(Bits { ty, data })
        }
    }
    /// Get this data
    pub fn data(&self) -> u128 {
        self.data
    }
    /// Get the (bits) type of this bitset
    pub fn get_ty(&self) -> VarRef<BitsTy> {
        self.ty.borrow_var()
    }
} 


debug_from_display!(Bits);
quick_pretty!(Bits, "Unimplemented!");
trivial_substitute!(Bits);
enum_convert! {
    impl InjectionRef<ValueEnum> for Bits {}
    impl TryFrom<NormalValue> for Bits { as ValueEnum, }
    impl TryFromRef<NormalValue> for Bits { as ValueEnum, }
}

impl Typed for Bits {
    #[inline]
    fn ty(&self) -> TypeRef {
        FIN.borrow_ty()
    }

    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Apply for Bits {}

impl Regional for Bits {}

impl Value for Bits {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Tried to get dependency #{} of bits {}, which has none",
            ix, self
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Bits(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Bits(self))
    }
}

impl From<Bits> for NormalValue {
    fn from(b: Bits) -> NormalValue {
        b.into_norm()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bitset_work() {
        let _data_1 = BitsTy(2).data(1).unwrap();
        let _data_2 = BitsTy(2).data(3).unwrap();
        assert!(BitsTy(2).data(4).is_err());

        let _data_3 = BitsTy(4).data(3).unwrap();
        assert!(BitsTy(4).data(16).is_err());

        let _data_4 = BitsTy(128).data(534567).unwrap();
    }
}