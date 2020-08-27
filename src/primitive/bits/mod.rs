/*!
Finite-valued types
*/
use crate::eval::{Application, Apply, EvalCtx};
use crate::function::pi::Pi;
use crate::region::{Region, Regional};
use crate::typing::{
    primitive::{Fin, Prop, FIN, SET},
    Kind, Type, Typed, Universe,
};
use crate::value::{
    Error, KindId, NormalValue, TypeId, TypeRef, UniverseId, UniverseRef, ValId, Value, ValueData,
    ValueEnum, VarId, VarRef,
};
use crate::{debug_from_display, enum_convert, quick_pretty, trivial_substitute, tyarr};
use lazy_static::lazy_static;
use num::ToPrimitive;
use ref_cast::RefCast;

mod add;
mod bits_impl;
mod div;
mod ext;
mod modl;
mod mul;
mod neg;
mod sh;
mod sub;

pub use add::*;
pub use mul::*;
pub use neg::*;
pub use sub::*;

lazy_static! {
    /// The kind of bits
    pub static ref BITS_KIND: VarId<BitsKind> = VarId::direct_new(BitsKind);
}

/// The type of types which are just a collection of bits, and hence can have bitwise operations performed on them
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BitsKind;

/// A type with `n` values
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, RefCast)]
#[repr(transparent)]
pub struct BitsTy(pub u32);

/// A bitvector constant
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Bits {
    /// The type of this bitvector
    ty: VarId<BitsTy>,
    /// This bitvector's data represented as a u128
    ///
    /// High order bits are assumed to be 0 by the hash-consing algorithm!
    data: u128,
    /// The length of this bitvector
    len: u32,
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

impl VarId<BitsTy> {
    /// Get a bitset into this type. Return an error if too many bits
    pub fn data<I: ToPrimitive>(&self, data: I) -> Result<Bits, Error> {
        let data = if let Some(data) = data.to_u128() {
            data
        } else {
            return Err(Error::TooManyBits);
        };
        Bits::try_new(self.clone(), data)
    }
    /// Get a bitset into this type. Return an error if too many bits
    pub fn into_data<I: ToPrimitive>(self, data: I) -> Result<Bits, Error> {
        let data = if let Some(data) = data.to_u128() {
            data
        } else {
            return Err(Error::TooManyBits);
        };
        Bits::try_new(self, data)
    }
}

impl Bits {
    /// Try to construct a new bitset. Return an error if high bits are set.
    pub fn try_new<B: Into<VarId<BitsTy>>>(ty: B, data: u128) -> Result<Bits, Error> {
        let ty: VarId<BitsTy> = ty.into();
        let len: u32 = ty.0;
        if len > 128 || data.wrapping_shr(len.min(127)) != 0 {
            Err(Error::TooManyBits)
        } else {
            Ok(Bits { ty, data, len })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing::primitive::FIN;

    #[test]
    fn bits_types_and_kind_work() {
        let bits_ty = BitsTy(72).into_var();
        let bits = bits_ty.data(36).unwrap();
        assert_eq!(bits.ty(), bits_ty);
        assert_eq!(bits_ty.ty(), *BITS_KIND);
        assert_eq!(bits.kind(), *BITS_KIND);
        assert_eq!(bits_ty.universe(), *FIN);
    }
    #[test]
    fn bitvector_construction_works() {
        let data_1 = BitsTy(2).data(1).unwrap();
        let data_2 = BitsTy(2).data(3).unwrap();
        assert!(BitsTy(2).data(4).is_err());
        assert_ne!(data_1, data_2, "Bitvector equality sanity test");

        let data_3 = BitsTy(4).data(3).unwrap();
        assert!(BitsTy(4).data(16).is_err());
        assert_ne!(
            data_2, data_3,
            "Bitvector equality should take length into account"
        );

        // Testing a literal which overflows a u64
        let _big_data = BitsTy(128).data(0x595643948456453445454512u128).unwrap();

        // Negative data fails
        assert!(BitsTy(23).data(-1).is_err());
    }
}
