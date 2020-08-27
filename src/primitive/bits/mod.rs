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
    #[test]
    fn constant_bitvector_addition_works() {
        let test_cases: &[(u32, u128, u128, u128)] = &[
            // Basic test cases, with no overflow
            (2, 1, 2, 3),
            (4, 3, 4, 7),
            // Overflow tests
            (2, 3, 3, 6 % 4),
            (7, 127, 1, 0),
            (8, 255, 1, 0),
        ];
        for (len, left, right, result) in test_cases.iter() {
            let left_data = BitsTy(*len).data(*left).expect("Left data is valid");
            let right_data = BitsTy(*len).data(*right).expect("Right data is valid");
            let add_struct = Add::new(*len);
            let data_arr = [left_data.into_val(), right_data.into_val()];
            let mut ctx = None;
            match add_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_add(*right)));
                        assert_eq!(b.data, masked_add(*len, *left, *right));
                        assert_eq!(b.data, add_struct.masked_add(*left, *right));
                    }
                    _ => panic!("Result should be a bitvector constant (ValueEnum::Bits)"),
                },
                _ => panic!("Application should not be symbolic"),
            };
            assert_eq!(
                ctx, None,
                "No evaluation context should be generated by direct addition of constants"
            );
        }
    }
    #[test]
    fn constant_bitvector_multiplication_works() {
        let test_cases: &[(u32, u128, u128, u128)] = &[
            // Basic test cases, with no overflow
            (2, 1, 2, 2),
            (4, 3, 4, 12),
            (4, 3, 1, 3),
            (4, 3, 0, 0),
            // Overflow tests
            (2, 3, 3, 9 % 4),
            (7, 127, 2, (127 * 2) % 128),
            (14, 8848, 2, 1312),
        ];
        for (len, left, right, result) in test_cases.iter() {
            let left_data = BitsTy(*len).data(*left).expect("Left data is valid");
            let right_data = BitsTy(*len).data(*right).expect("Right data is valid");
            let multiply_struct = Mul::new(*len);
            let data_arr = [left_data.into_val(), right_data.into_val()];
            let mut ctx = None;
            match multiply_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_mul(*right)));
                        assert_eq!(b.data, masked_mul(*len, *left, *right));
                        assert_eq!(b.data, multiply_struct.masked_mul(*left, *right));
                    }
                    _ => panic!("Result should be a bitvector constant (ValueEnum::Bits)"),
                },
                _ => panic!("Application should not be symbolic"),
            };
            assert_eq!(
                ctx, None,
                "No evaluation context should be generated by direct addition of constants"
            );
        }
    }
    #[test]
    fn constant_bitvector_substraction_works() {
        let test_cases: &[(u32, u128, u128, u128)] = &[
            // Basic test cases, with no underflow
            (2, 2, 1, 1),
            (4, 12, 9, 3),
            (14, 8848, 0, 8848),
            // underflow tests
            (4, 3, 4, 15),
            (8, 253, 255, 254),
        ];
        for (len, left, right, result) in test_cases.iter() {
            let left_data = BitsTy(*len).data(*left).expect("Left data is valid");
            let right_data = BitsTy(*len).data(*right).expect("Right data is valid");
            let subtract_struct = Sub::new(*len);
            let data_arr = [left_data.into_val(), right_data.into_val()];
            let mut ctx = None;
            match subtract_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_sub(*right)));
                        assert_eq!(b.data, masked_sub(*len, *left, *right));
                        assert_eq!(b.data, subtract_struct.masked_sub(*left, *right));
                    }
                    _ => panic!("Result should be a bitvector constant (ValueEnum::Bits)"),
                },
                _ => panic!("Application should not be symbolic"),
            };
            assert_eq!(
                ctx, None,
                "No evaluation context should be generated by direct addition of constants"
            );
        }
    }
    #[test]
    fn constant_bitvector_negation_works() {
        let test_cases: &[(u32, u128, u128)] = &[(4, 2, 14), (4, 0, 0), (10, 1, 1023)];
        for (len, data, result) in test_cases.iter() {
            let op_data = BitsTy(*len).data(*data).expect("data is valid");
            let neg_struct = Neg::new(*len);
            let data_arr = [op_data.into_val()];
            let mut ctx = None;
            match neg_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, data.wrapping_neg()));
                        assert_eq!(b.data, masked_neg(*len, *data));
                        assert_eq!(b.data, neg_struct.masked_neg(*data));
                    }
                    _ => panic!("Result should be a bitvector constant (ValueEnum::Bits)"),
                },
                _ => panic!("Application should not be symbolic"),
            };
            assert_eq!(
                ctx, None,
                "No evaluation context should be generated by direct addition of constants"
            );
        }
    }
}
