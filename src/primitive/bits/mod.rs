/*!
Finite-valued types
*/
use crate::eval::{Application, Apply, EvalCtx};
use crate::function::{lambda::Lambda, pi::Pi};
use crate::primitive::logical::Bool;
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
use std::ops::Index;

mod bits_impl;
mod div;
mod ext;
mod modl;
mod neg;
mod sh;

pub use neg::*;

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

impl Index<u32> for Bits {
    type Output = bool;
    #[inline(always)]
    fn index(&self, n: u32) -> &bool {
        if self.bit(n) {
            &true
        } else {
            &false
        }
    }
}

/// Bitvector operations
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum BinOp {
    /// Bitvector addition
    Add,
    /// Bitvector subtraction
    Sub,
    /// Bitvector modulo, where we take n % 0 = n
    Mod,
    /// Bitvector multiplication
    Mul,
}

impl BinOp {
    /// Return the right identity of this operation
    fn right_identity(&self) -> Option<u128> {
        match self {
            BinOp::Add | BinOp::Sub | BinOp::Mod => Some(0),
            BinOp::Mul => Some(1),
        }
    }
    /// Return the right identity of this operation
    fn left_identity(&self) -> Option<u128> {
        match self {
            BinOp::Add => Some(0),
            BinOp::Sub => None,
            BinOp::Mod => None,
            BinOp::Mul => Some(1),
        }
    }
    /// Return the right opreand for which the result is always 0
    fn right_sink(&self) -> Option<u128> {
        match self {
            BinOp::Add | BinOp::Sub => None,
            BinOp::Mod => Some(1),
            BinOp::Mul => Some(0),
        }
    }
    /// Return the left opreand for which the result is always 0
    fn left_sink(&self) -> Option<u128> {
        match self {
            BinOp::Mul | BinOp::Mod => Some(0),
            _ => None,
        }
    }
}

debug_from_display!(BinOp);
quick_pretty!(BinOp, "#BinOp");
trivial_substitute!(BinOp);
enum_convert! {
    impl InjectionRef<ValueEnum> for BinOp {}
    impl TryFrom<NormalValue> for BinOp { as ValueEnum, }
    impl TryFromRef<NormalValue> for BinOp { as ValueEnum, }
}

impl From<BinOp> for NormalValue {
    fn from(a: BinOp) -> NormalValue {
        a.into_norm()
    }
}

impl Regional for BinOp {}

impl Apply for BinOp {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        if args.len() == 2 {
            if let ValueEnum::Bits(b) = args[1].as_enum() {
                if b.ty != args[0] {
                    return Err(Error::TypeMismatch);
                }
                if b.data == 1 {
                    return Ok(Application::Success(
                        &[],
                        Lambda::id(args[0].clone().coerce()).into_val(),
                    ));
                }
            }
        }
        if args.len() <= 2 {
            BITS_BINARY
                .apply_ty_in(args, ctx)
                .map(Application::Symbolic)
        } else {
            match (args[0].as_enum(), args[1].as_enum(), args[2].as_enum()) {
                (ValueEnum::BitsTy(ty), ValueEnum::Bits(left), ValueEnum::Bits(right)) => {
                    if left.len != right.len || left.len != ty.0 {
                        return Err(Error::TypeMismatch);
                    }
                    let data = match self {
                        BinOp::Add => masked_add(ty.0, left.data, right.data),
                        BinOp::Sub => masked_sub(ty.0, left.data, right.data),
                        BinOp::Mod => unimplemented!("Modulo is nor implemented"),
                        BinOp::Mul => masked_mul(ty.0, left.data, right.data),
                    };
                    let result = Bits {
                        ty: left.ty.clone(),
                        data,
                        len: left.len,
                    };
                    result.apply_in(&args[3..], ctx)
                }
                // Right sinks to zero
                (ValueEnum::BitsTy(ty), x, ValueEnum::Bits(zero))
                    if self.right_sink().is_some() && zero.data == self.right_sink().unwrap() =>
                {
                    if zero.len != ty.0 || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    args[2].apply_in(&args[3..], ctx)
                }
                // Left sinks to zero
                (ValueEnum::BitsTy(ty), ValueEnum::Bits(zero), x)
                    if self.left_sink().is_some() && zero.data == self.left_sink().unwrap() =>
                {
                    if zero.len != ty.0 || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    args[1].apply_in(&args[3..], ctx)
                }
                // Left identity
                (ValueEnum::BitsTy(ty), ValueEnum::Bits(one), x)
                    if self.left_identity().is_some()
                        && one.data == self.left_identity().unwrap() =>
                {
                    if one.len != ty.0 || one.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    args[2].apply_in(&args[3..], ctx)
                }
                // Right identity
                (ValueEnum::BitsTy(ty), x, ValueEnum::Bits(one))
                    if self.right_identity().is_some()
                        && one.data == self.right_identity().unwrap() =>
                {
                    if one.len != ty.0 || one.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    args[1].apply_in(&args[3..], ctx)
                }
                (ty, left, right) => {
                    if ty.ty() != *BITS_KIND {
                        return Err(Error::TypeMismatch);
                    }
                    let left_ty = left.ty();
                    if left_ty != right.ty() || left_ty != args[0] || ty.ty() != *BITS_KIND {
                        Err(Error::TypeMismatch)
                    } else {
                        left_ty
                            .apply_ty_in(&args[3..], ctx)
                            .map(Application::Symbolic)
                    }
                }
            }
        }
    }
}

impl Typed for BinOp {
    #[inline]
    fn ty(&self) -> TypeRef {
        BITS_BINARY.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
    #[inline]
    fn is_kind(&self) -> bool {
        false
    }
}

impl Type for BinOp {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Value for BinOp {
    fn no_deps(&self) -> usize {
        0
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "BinOp operation {} has no dependencies (tried to get dep #{})",
            self, ix
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::BinOp(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::BinOp(self))
    }
}

impl ValueData for BinOp {}

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

#[inline(always)]
fn masked_add(len: u32, left: u128, right: u128) -> u128 {
    mask(len, left.wrapping_add(right))
}

#[inline(always)]
fn masked_mul(len: u32, left: u128, right: u128) -> u128 {
    mask(len, left.wrapping_mul(right))
}

#[inline(always)]
fn masked_sub(len: u32, left: u128, right: u128) -> u128 {
    mask(len, left.wrapping_sub(right))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::finite::Finite;
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
    fn bitvector_construction_and_application_work() {
        let data_1 = BitsTy(2).data(1).unwrap();
        let data_2 = BitsTy(2).data(3).unwrap();
        assert!(BitsTy(2).data(4).is_err());
        assert_ne!(data_1, data_2, "Bitvector equality sanity test");
        assert!(data_1.bit(0));
        assert!(!data_1.bit(1));
        assert!(data_2.bit(0));
        assert!(data_2.bit(1));
        assert!(data_1[0]);
        assert!(!data_1[1]);
        assert!(data_2[0]);
        assert!(data_2[1]);
        assert!(!data_1.bit_zext(2));
        assert!(!data_2.bit_zext(2));
        assert_eq!(data_1.try_bit(2), Err(()));
        assert_eq!(data_2.try_bit(2), Err(()));
        assert_eq!(data_1.len(), 2);
        assert_eq!(data_2.len(), 2);

        let finite_2 = Finite(2).into_var();
        let ix_0 = &[finite_2.ix(0).unwrap().into_val()][..];
        let ix_1 = &[finite_2.ix(1).unwrap().into_val()][..];
        assert_eq!(data_1.applied(ix_0), Ok(true.into_val()));
        assert_eq!(data_1.applied(ix_1), Ok(false.into_val()));
        assert_eq!(data_2.applied(ix_0), Ok(true.into_val()));
        assert_eq!(data_2.applied(ix_1), Ok(true.into_val()));
        assert!(data_1
            .applied(std::slice::from_ref(finite_2.as_val()))
            .is_err());

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
    fn constant_bitvector_subtraction_works() {
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
            let bitwidth = BitsTy(*len).into_var();
            let left_data = bitwidth.data(*left).expect("Left data is valid");
            let right_data = bitwidth.data(*right).expect("Right data is valid");
            let data_arr = [
                bitwidth.into_val(),
                left_data.into_val(),
                right_data.into_val(),
            ];
            let mut ctx = None;
            let op_struct = BinOp::Sub;
            match op_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_sub(*right)));
                        assert_eq!(b.data, masked_sub(*len, *left, *right));
                    }
                    _ => panic!("Result should be a bitvector constant (ValueEnum::Bits)"),
                },
                _ => panic!("Application should not be symbolic"),
            };
            assert_eq!(
                ctx, None,
                "No evaluation context should be generated by direct subtraction of constants"
            );
        }
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
            let bitwidth = BitsTy(*len).into_var();
            let left_data = bitwidth.data(*left).expect("Left data is valid");
            let right_data = bitwidth.data(*right).expect("Right data is valid");
            let data_arr = [
                bitwidth.into_val(),
                left_data.into_val(),
                right_data.into_val(),
            ];
            let mut ctx = None;
            let op_struct = BinOp::Add;
            match op_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_add(*right)));
                        assert_eq!(b.data, masked_add(*len, *left, *right));
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

    //FIXME: this!
    /*
    #[test]
    fn bitvector_addition_does_not_accept_invalid_types() {
        Add.applied(&[Bool.into_val()])
            .expect_err("Booleans are not valid additive types");
        Add.applied(&[Bool.into_val(), true.into_val(), false.into_val()])
            .expect_err("Booleans are not valid additive types");
    }
    */

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
            let bitwidth = BitsTy(*len).into_var();
            let left_data = bitwidth.data(*left).expect("Left data is valid");
            let right_data = bitwidth.data(*right).expect("Right data is valid");
            let data_arr = [
                bitwidth.into_val(),
                left_data.into_val(),
                right_data.into_val(),
            ];
            let mut ctx = None;
            let op_struct = BinOp::Mul;
            // Complete application
            match op_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_mul(*right)));
                        assert_eq!(b.data, masked_mul(*len, *left, *right));
                    }
                    _ => panic!("Result should be a bitvector constant (ValueEnum::Bits)"),
                },
                _ => panic!("Application should not be symbolic"),
            };
            assert_eq!(
                ctx, None,
                "No evaluation context should be generated by direct multiplication of constants"
            );
            /*
            // Nested partial application
            match Mul
                .applied_in(&data_arr[..2], &mut ctx)
                .unwrap()
                .applied_in(&data_arr[2..], &mut ctx)
                .unwrap()
                .as_enum()
            {
                ValueEnum::Bits(b) => {
                    assert_eq!(b.len, *len);
                    assert_eq!(b.data, *result);
                    assert_eq!(b.data, mask(*len, left.wrapping_mul(*right)));
                    assert_eq!(b.data, masked_mul(*len, *left, *right));
                }
                r => panic!("Result should be a bitvector constant (ValueEnum::Bits), but got {}", r),
            };
            */
        }
    }
}
