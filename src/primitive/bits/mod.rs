/*!
Finite-valued types
*/
use crate::eval::{Application, Apply, EvalCtx};
use crate::function::{lambda::Lambda, pi::Pi};
use crate::region::{Parameter, Region, Regional};
use crate::typing::{
    primitive::{Prop, Set, FIN, SET},
    Kind, Type, Typed, Universe,
};
use crate::value::{
    arr::TyArr, Error, KindId, NormalValue, TypeId, TypeRef, UniverseId, UniverseRef, ValId, Value,
    ValueData, ValueEnum, VarId, VarRef,
};
use crate::{debug_from_display, enum_convert, quick_pretty, trivial_substitute};
use lazy_static::lazy_static;
use num::ToPrimitive;
use ref_cast::RefCast;
use std::iter::once;

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
    /// The type array containing only the bits kind
    pub static ref BITS_ARR: TyArr = BitsKind::compute_bits_arr();
    /// The region with a bits kind as paremeter
    pub static ref BITS_REGION: Region = BitsKind::compute_bits_region();
    /// The type parameter of the bits region
    pub static ref BITS_PARAM: VarId<Parameter> = BitsKind::compute_bits_param();
    /// The type parameter of the bits region as a guaranteed type
    pub static ref BITS_PARAM_TY: TypeId = BITS_PARAM.clone().try_into_ty().unwrap();
    /// The kind of binary operators on bits
    pub static ref BITS_BINARY: VarId<Pi> = BitsKind::compute_binary_ty().into_var();
    /// The kind of unary operators on bits
    pub static ref BITS_UNARY: VarId<Pi> = BitsKind::compute_unary_ty().into_var();
}

/// The type of types which are just a collection of bits, and hence can have bitwise operations performed on them
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BitsKind;

impl BitsKind {
    /// Compute the bits array
    fn compute_bits_arr() -> TyArr {
        once(BITS_KIND.clone_as_ty()).collect()
    }
    /// Compute the bits region
    fn compute_bits_region() -> Region {
        Region::with_unchecked(
            BITS_ARR.clone(),
            Region::NULL,
            Set::default().into_universe(),
        )
    }
    /// Compute the type parameter of the bits region
    fn compute_bits_param() -> VarId<Parameter> {
        BITS_REGION.param(0).unwrap().into_var()
    }
    /// Compute the type of unary operators parametric over `BitsKind`
    fn compute_unary_ty() -> Pi {
        let variable_width_ty = Pi::unary(BITS_PARAM_TY.clone()).into_ty();
        Pi::try_new(variable_width_ty, BITS_REGION.clone())
            .expect("The type of the addition operator is always valid")
    }
    /// Compute the type of binary operators parametric over `BitsKind`
    fn compute_binary_ty() -> Pi {
        let variable_width_ty = Pi::binary(BITS_PARAM_TY.clone()).into_ty();
        Pi::try_new(variable_width_ty, BITS_REGION.clone())
            .expect("The type of the addition operator is always valid")
    }
}

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
    /// Try to construct a new bitvector. Return an error if high bits are set.
    pub fn try_new<B: Into<VarId<BitsTy>>>(ty: B, data: u128) -> Result<Bits, Error> {
        let ty: VarId<BitsTy> = ty.into();
        let len: u32 = ty.0;
        if len > 128 || data.wrapping_shr(len.min(127)) != 0 {
            Err(Error::TooManyBits)
        } else {
            Ok(Bits { ty, data, len })
        }
    }
    /// Get the data of this bitvector
    #[inline(always)]
    pub fn data(&self) -> u128 {
        self.data
    }
    /// Get the (bits) type of this bitvector
    #[inline(always)]
    pub fn get_ty(&self) -> VarRef<BitsTy> {
        self.ty.borrow_var()
    }
    /// Get the `n`th bit of a bitvector. Return an error on out of bounds
    #[inline(always)]
    pub fn try_bit(&self, n: u32) -> Result<bool, ()> {
        if n >= self.len {
            Err(())
        } else {
            Ok(self.data & 1u128.wrapping_shl(n) != 0)
        }
    }
    /// Get the `n`th bit of a bitvector, panicking on out of bounds
    #[inline(always)]
    pub fn bit(&self, n: u32) -> bool {
        self.try_bit(n).expect("Valid bit index")
    }
    /// Get the `n`th bit of a bitvector, zero extending on out of bounds
    #[inline(always)]
    pub fn bit_zext(&self, n: u32) -> bool {
        self.data & 1u128.wrapping_shl(n) != 0
    }
    /// Get the length of a bitvector
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.len
    }
    /// Get whether this bitvector is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Mask a bitvector, discarding bits of order greater than `len`
///
/// # Examples
/// ```rust
/// # use rain_ir::primitive::bits::mask;
/// assert_eq!(mask(3, 0b11100010100011100101), 0b101);
/// ```
#[inline(always)]
pub fn mask(len: u32, vector: u128) -> u128 {
    let len = len.min(128);
    vector.wrapping_shl(128 - len).wrapping_shr(128 - len)
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
        assert!(data_1.bit(0));
        assert!(!data_1.bit(1));
        assert!(data_2.bit(0));
        assert!(data_2.bit(1));
        assert!(!data_1.bit_zext(2));
        assert!(!data_2.bit_zext(2));
        assert_eq!(data_1.try_bit(2), Err(()));
        assert_eq!(data_2.try_bit(2), Err(()));
        assert_eq!(data_1.len(), 2);
        assert_eq!(data_2.len(), 2);

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
