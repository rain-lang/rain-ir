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
                        data: left.data.wrapping_add(right.data),
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

impl Add {
    /// Try to initilize an add operator for #bit(len)
    pub fn new(len: u32) -> Add {
        let region = Region::with_unchecked(
            tyarr![BitsTy{0: len}.into_ty(); 2],
            Region::NULL,
            Fin.into_universe(),
        );
        let pi = Pi::try_new(BitsTy { 0: len }.into_ty(), region)
            .unwrap()
            .into_var();
        Add { ty: pi, len }
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
    fn bitset_work() {
        let _data_1 = BitsTy(2).data(1).unwrap();
        let _data_2 = BitsTy(2).data(3).unwrap();
        assert!(BitsTy(2).data(4).is_err());

        let _data_3 = BitsTy(4).data(3).unwrap();
        assert!(BitsTy(4).data(16).is_err());

        let _data_4 = BitsTy(128).data(534567).unwrap();
    }
    #[test]
    fn bitset_add_work() {
        let test_cases: [(u32, u128, u128); 2] = [(2, 1, 2), (4, 3, 4)];
        for (len, num_1, num_2) in test_cases.iter() {
            let data_1 = BitsTy(*len).data(*num_1).unwrap();
            let data_2 = BitsTy(*len).data(*num_2).unwrap();
            let add_struct = Add::new(*len);
            let data_arr = [data_1.into_val(), data_2.into_val()];
            match add_struct.apply_in(&data_arr[..], &mut None).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                    }
                    _ => panic!("Returned result should be a Bits value"),
                },
                _ => panic!("Should be a Application::Success"),
            };
        }
    }
}
