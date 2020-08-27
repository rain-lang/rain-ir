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
mod sh;
mod sub;

pub use add::*;

/// The type of types which are just a collection of bits, and hence can have bitwise operations performed on them
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BitsKind;

debug_from_display!(BitsKind);
quick_pretty!(BitsKind, "#bitskind");
trivial_substitute!(BitsKind);
enum_convert! {
    impl InjectionRef<ValueEnum> for BitsKind {}
    impl TryFrom<NormalValue> for BitsKind { as ValueEnum, }
    impl TryFromRef<NormalValue> for BitsKind { as ValueEnum, }
}

impl ValueData for BitsKind {}

impl Typed for BitsKind {
    #[inline]
    fn ty(&self) -> TypeRef {
        SET.borrow_ty()
    }

    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
    #[inline]
    fn is_kind(&self) -> bool {
        true
    }
}

impl Apply for BitsKind {}

impl Regional for BitsKind {}

impl Value for BitsKind {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Tried to get dependency #{} of bits kind, which has none",
            ix
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::BitsKind(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::BitsKind(self))
    }
}

impl From<BitsKind> for NormalValue {
    fn from(b: BitsKind) -> NormalValue {
        b.into_norm()
    }
}

impl Type for BitsKind {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Kind for BitsKind {
    fn id_kind(&self) -> KindId {
        Prop.into_kind()
    }
    fn closure(&self) -> UniverseId {
        Prop.into_universe()
    }
    fn try_closure(&self) -> Option<UniverseRef> {
        Some(FIN.borrow_universe())
    }
}

lazy_static! {
    /// The kind of bits
    pub static ref BITS_KIND: VarId<BitsKind> = VarId::direct_new(BitsKind);
}

/// A type with `n` values
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, RefCast)]
#[repr(transparent)]
pub struct BitsTy(pub u32);

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

impl ValueData for BitsTy {}

impl Typed for BitsTy {
    #[inline]
    fn ty(&self) -> TypeRef {
        BITS_KIND.borrow_ty()
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
    /// The length of this bitvector
    len: u32,
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
        self.ty.borrow_ty()
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

/// The multiplication operator
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Mul {
    /// Type of the multiply operator
    ty: VarId<Pi>,
    /// The length of the bit vector,
    len: u32,
}

#[allow(clippy::len_without_is_empty)]
impl Mul {
    /// Create a multiply operator with bitwidth `len`
    pub fn new(len: u32) -> Mul {
        Mul {
            ty: Self::compute_ty(len).into_var(),
            len,
        }
    }
    /// Get the pi type of the multiplication operator with bitwidth `len`
    ///
    /// Note that the result of this method called on `len` is always equal to the type of `Mul::new(len)`.
    pub fn compute_ty(len: u32) -> Pi {
        let region = Region::with_unchecked(
            tyarr![BitsTy{0: len}.into_ty(); 2],
            Region::NULL,
            Fin.into_universe(),
        );
        Pi::try_new(BitsTy(len).into_ty(), region)
            .expect("The type of the multiplication operator is always valid")
    }
    /// Perform wrapping bitvector multiplication, discarding high order bits
    ///
    /// This method assumes both `left` and `right` are valid bitvectors for this multiplication operation, namely that they have length
    /// less than or equal to `self.len()`. If this is not the case, this function will panic *in debug mode*, while in release mode,
    /// the behaviour is unspecified but safe.
    #[inline(always)]
    pub fn masked_mul(&self, left: u128, right: u128) -> u128 {
        debug_assert_eq!(
            left,
            mask(self.len, left),
            "Left bitvector has length greater than len"
        );
        debug_assert_eq!(
            right,
            mask(self.len, right),
            "Right bitvector has length greater than len"
        );
        masked_mul(self.len, left, right)
    }
    /// Get the bitwidth of this addition operator
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.len
    }
}

/// Perform wrapping bitvector multiplication, discarding bits of order greater than `len`
#[inline(always)]
pub fn masked_mul(len: u32, left: u128, right: u128) -> u128 {
    mask(len, left.wrapping_mul(right))
}

debug_from_display!(Mul);
quick_pretty!(Mul, "Mul(Need to change this)");
trivial_substitute!(Mul);
enum_convert! {
    impl InjectionRef<ValueEnum> for Mul {}
    impl TryFrom<NormalValue> for Mul { as ValueEnum, }
    impl TryFromRef<NormalValue> for Mul { as ValueEnum, }
}

impl From<Mul> for NormalValue {
    fn from(a: Mul) -> NormalValue {
        a.into_norm()
    }
}

impl Regional for Mul {}

impl Apply for Mul {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        if args.len() <= 1 {
            self.ty.apply_ty_in(args, ctx).map(Application::Symbolic)
        } else if args.len() > 2 {
            Err(Error::TooManyArgs)
        } else {
            let arg_0 = &args[0];
            let arg_1 = &args[1];
            match (arg_0.as_enum(), arg_1.as_enum()) {
                (ValueEnum::Bits(left), ValueEnum::Bits(right)) => {
                    if left.len != right.len || left.len != self.len {
                        return Err(Error::TypeMismatch);
                    }
                    let result = Bits {
                        ty: left.ty.clone(),
                        data: self.masked_mul(left.data, right.data),
                        len: left.len,
                    };
                    Ok(Application::Success(&[], result.into_val()))
                }
                (ValueEnum::Bits(zero), x) | (x, ValueEnum::Bits(zero)) if zero.data == 0 => {
                    if zero.len != self.len || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    let result = BitsTy { 0: zero.len }.data(0).unwrap();
                    Ok(Application::Success(&[], result.into_val()))
                }
                (left, right) => {
                    let left_ty = left.ty();
                    if left_ty != right.ty() {
                        // Error
                        return Err(Error::TypeMismatch);
                    }
                    match left {
                        ValueEnum::BitsTy(b) if b.0 != self.len => {
                            Ok(Application::Symbolic(left_ty.clone_as_ty()))
                        }
                        _ => Err(Error::TypeMismatch),
                    }
                }
            }
        }
    }
}

impl Typed for Mul {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
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

impl Type for Mul {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Value for Mul {
    fn no_deps(&self) -> usize {
        0
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Add operation {} has no dependencies (tried to get dep #{})",
            self, ix
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Mul(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Mul(self))
    }
}

impl ValueData for Mul {}

/// The subtraction operator
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Subtract {
    /// Type of the subtraction operator
    ty: VarId<Pi>,
    /// The length of the bit vector,
    len: u32,
}

#[allow(clippy::len_without_is_empty)]
impl Subtract {
    /// Create an substration operator with bitwidth `len`
    pub fn new(len: u32) -> Subtract {
        Subtract {
            ty: Self::compute_ty(len).into_var(),
            len,
        }
    }
    /// Get the pi type of the substration operator with bitwidth `len`
    ///
    /// Note that the result of this method called on `len` is always equal to the type of `Subtract::new(len)`.
    pub fn compute_ty(len: u32) -> Pi {
        let region = Region::with_unchecked(
            tyarr![BitsTy{0: len}.into_ty(); 2],
            Region::NULL,
            Fin.into_universe(),
        );
        Pi::try_new(BitsTy(len).into_ty(), region)
            .expect("The type of the multiply operator is always valid")
    }
    /// Perform wrapping bitvector substraction, discarding high order bits
    ///
    /// This method assumes both `left` and `right` are valid bitvectors for this substraction operation, namely that they have length
    /// less than or equal to `self.len()`. If this is not the case, this function will panic *in debug mode*, while in release mode,
    /// the behaviour is unspecified but safe.
    #[inline(always)]
    pub fn masked_subtract(&self, left: u128, right: u128) -> u128 {
        debug_assert_eq!(
            left,
            mask(self.len, left),
            "Left bitvector has length greater than len"
        );
        debug_assert_eq!(
            right,
            mask(self.len, right),
            "Right bitvector has length greater than len"
        );
        masked_subtract(self.len, left, right)
    }
    /// Get the bitwidth of this subtract operator
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.len
    }
}

/// Perform wrapping bitvector multiplication, discarding bits of order greater than `len`
#[inline(always)]
pub fn masked_subtract(len: u32, left: u128, right: u128) -> u128 {
    mask(len, left.wrapping_sub(right))
}

debug_from_display!(Subtract);
quick_pretty!(Subtract, "Mul(Need to change this)");
trivial_substitute!(Subtract);
enum_convert! {
    impl InjectionRef<ValueEnum> for Subtract {}
    impl TryFrom<NormalValue> for Subtract { as ValueEnum, }
    impl TryFromRef<NormalValue> for Subtract { as ValueEnum, }
}

impl From<Subtract> for NormalValue {
    fn from(a: Subtract) -> NormalValue {
        a.into_norm()
    }
}

impl Regional for Subtract {}

impl Apply for Subtract {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        if args.len() <= 1 {
            self.ty.apply_ty_in(args, ctx).map(Application::Symbolic)
        } else if args.len() > 2 {
            Err(Error::TooManyArgs)
        } else {
            let arg_0 = &args[0];
            let arg_1 = &args[1];
            match (arg_0.as_enum(), arg_1.as_enum()) {
                (ValueEnum::Bits(left), ValueEnum::Bits(right)) => {
                    if left.len != right.len || left.len != self.len {
                        return Err(Error::TypeMismatch);
                    }
                    let result = Bits {
                        ty: left.ty.clone(),
                        data: self.masked_subtract(left.data, right.data),
                        len: left.len,
                    };
                    Ok(Application::Success(&[], result.into_val()))
                }
                (left, right) => {
                    let left_ty = left.ty();
                    if left_ty != right.ty() {
                        // Error
                        return Err(Error::TypeMismatch);
                    }
                    match left {
                        ValueEnum::BitsTy(b) if b.0 != self.len => {
                            Ok(Application::Symbolic(left_ty.clone_as_ty()))
                        }
                        _ => Err(Error::TypeMismatch),
                    }
                }
            }
        }
    }
}

impl Typed for Subtract {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
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

impl Type for Subtract {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Value for Subtract {
    fn no_deps(&self) -> usize {
        0
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Add operation {} has no dependencies (tried to get dep #{})",
            self, ix
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Subtract(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Subtract(self))
    }
}

impl ValueData for Subtract {}

/// The negation operator
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Neg {
    /// Type of the negation operator
    ty: VarId<Pi>,
    /// The length of the bit vector,
    len: u32,
}

#[allow(clippy::len_without_is_empty)]
impl Neg {
    /// Create an negation operator with bitwidth `len`
    pub fn new(len: u32) -> Neg {
        Neg {
            ty: Self::compute_ty(len).into_var(),
            len,
        }
    }
    /// Get the pi type of the negation operator with bitwidth `len`
    ///
    /// Note that the result of this method called on `len` is always equal to the type of `Mul::new(len)`.
    pub fn compute_ty(len: u32) -> Pi {
        let region = Region::with_unchecked(
            tyarr![BitsTy{0: len}.into_ty(); 2],
            Region::NULL,
            Fin.into_universe(),
        );
        Pi::try_new(BitsTy(len).into_ty(), region)
            .expect("The type of the multiply operator is always valid")
    }
    /// Perform wrapping bitvector negation, discarding high order bits
    ///
    /// This method assumes both `b` are valid bitvectors for this negation operation, namely that they have length
    /// less than or equal to `self.len()`. If this is not the case, this function will panic *in debug mode*, while in release mode,
    /// the behaviour is unspecified but safe.
    #[inline(always)]
    pub fn masked_neg(&self, b: u128) -> u128 {
        debug_assert_eq!(
            b,
            mask(self.len, b),
            "Bitvector for negation has length greater than len"
        );
        masked_neg(self.len, b)
    }
    /// Get the bitwidth of this addition operator
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.len
    }
}

/// Perform bitvector negation, discarding bits of order greater than `len`
#[inline(always)]
pub fn masked_neg(len: u32, b: u128) -> u128 {
    mask(len, b.wrapping_neg())
}

debug_from_display!(Neg);
quick_pretty!(Neg, "Mul(Need to change this)");
trivial_substitute!(Neg);
enum_convert! {
    impl InjectionRef<ValueEnum> for Neg {}
    impl TryFrom<NormalValue> for Neg { as ValueEnum, }
    impl TryFromRef<NormalValue> for Neg { as ValueEnum, }
}

impl From<Neg> for NormalValue {
    fn from(a: Neg) -> NormalValue {
        a.into_norm()
    }
}

impl Regional for Neg {}

impl Apply for Neg {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        if args.is_empty() {
            self.ty.apply_ty_in(args, ctx).map(Application::Symbolic)
        } else if args.len() > 1 {
            Err(Error::TooManyArgs)
        } else {
            let arg = &args[0];
            match arg.as_enum() {
                ValueEnum::Bits(b) => {
                    let result = Bits {
                        ty: b.ty.clone(),
                        data: self.masked_neg(b.data),
                        len: b.len,
                    };
                    Ok(Application::Success(&[], result.into_val()))
                }
                ValueEnum::BitsTy(b) if b.0 == self.len => {
                    Ok(Application::Symbolic(b.ty().clone_ty()))
                }
                _ => Err(Error::TypeMismatch),
            }
        }
    }
}

impl Typed for Neg {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
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

impl Type for Neg {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Value for Neg {
    fn no_deps(&self) -> usize {
        0
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Add operation {} has no dependencies (tried to get dep #{})",
            self, ix
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Neg(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Neg(self))
    }
}

impl ValueData for Neg {}

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
            let subtract_struct = Subtract::new(*len);
            let data_arr = [left_data.into_val(), right_data.into_val()];
            let mut ctx = None;
            match subtract_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_sub(*right)));
                        assert_eq!(b.data, masked_subtract(*len, *left, *right));
                        assert_eq!(b.data, subtract_struct.masked_subtract(*left, *right));
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
