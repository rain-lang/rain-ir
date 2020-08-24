/*!
Finite-valued types
*/
use crate::eval::{Application, Apply, EvalCtx};
use crate::function::pi::Pi;
use crate::region::{Region, Regional};
use crate::typing::{
    primitive::{Fin, FIN},
    Type, Typed, Universe,
};
use crate::value::{
    Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueData, ValueEnum, VarId, VarRef,
};
use crate::{debug_from_display, enum_convert, quick_pretty, trivial_substitute, tyarr};
use num::ToPrimitive;
use ref_cast::RefCast;

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

/// The add operator
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Add {
    /// Type of the add operator
    ty: VarId<Pi>,
    /// The length of the bit vector,
    len: u32,
}

#[allow(clippy::len_without_is_empty)]
impl Add {
    /// Create an addition operator with bitwidth `len`
    pub fn new(len: u32) -> Add {
        Add {
            ty: Self::compute_ty(len).into_var(),
            len,
        }
    }
    /// Get the pi type of the addition operator with bitwidth `len`
    /// 
    /// Note that the result of this method called on `len` is always equal to the type of `Add::new(len)`.
    pub fn compute_ty(len: u32) -> Pi {
        let region = Region::with_unchecked(
            tyarr![BitsTy{0: len}.into_ty(); 2],
            Region::NULL,
            Fin.into_universe(),
        );
        Pi::try_new(BitsTy(len).into_ty(), region)
            .expect("The type of the addition operator is always valid")
    }
    /// Perform wrapping bitvector addition, discarding high order bits
    ///
    /// This method assumes both `left` and `right` are valid bitvectors for this addition operation, namely that they have length
    /// less than or equal to `self.len()`
    #[inline(always)]
    pub fn masked_add(&self, left: u128, right: u128) -> u128 {
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
        masked_add(self.len, left, right)
    }
    /// Get the bitwidth of this addition operator
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.len
    }
}

/// Mask a bitvector, discarding bits of order greater than `len`
#[inline(always)]
pub fn mask(len: u32, vector: u128) -> u128 {
    let len = len.min(128);
    vector.wrapping_shl(128 - len).wrapping_shr(128 - len)
}

/// Perform wrapping bitvector addition, discarding bits of order greater than `len`
#[inline(always)]
pub fn masked_add(len: u32, left: u128, right: u128) -> u128 {
    mask(len, left.wrapping_add(right))
}

debug_from_display!(Add);
quick_pretty!(Add, "Add(Need to change this)");
trivial_substitute!(Add);
enum_convert! {
    impl InjectionRef<ValueEnum> for Add {}
    impl TryFrom<NormalValue> for Add { as ValueEnum, }
    impl TryFromRef<NormalValue> for Add { as ValueEnum, }
}

impl From<Add> for NormalValue {
    fn from(a: Add) -> NormalValue {
        a.into_norm()
    }
}

impl Regional for Add {}

impl Apply for Add {
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
                        data: self.masked_add(left.data, right.data),
                        len: left.len,
                    };
                    Ok(Application::Success(&[], result.into_val()))
                }
                (ValueEnum::Bits(zero), x) if zero.data == 0 => {
                    if zero.len != self.len || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    Ok(Application::Success(&[], args[0].clone()))
                }
                (x, ValueEnum::Bits(zero)) if zero.data == 0 => {
                    if zero.len != self.len || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    Ok(Application::Success(&[], args[1].clone()))
                }
                (left, right) => {
                    let left_ty = left.ty();
                    if left_ty != right.ty() {
                        // Error
                        return Err(Error::TypeMismatch);
                    }
                    match left {
                        ValueEnum::BitsTy(b) if b.0 != self.len => {
                            Ok(Application::Symbolic(left_ty.clone_ty()))
                        }
                        _ => Err(Error::TypeMismatch),
                    }
                }
            }
        }
    }
}

impl Typed for Add {
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

impl Type for Add {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Value for Add {
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
        ValueEnum::Add(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Add(self))
    }
}

impl ValueData for Add {}

#[cfg(test)]
mod tests {
    use super::*;
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
}
