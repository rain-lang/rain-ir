/*!
Primitive `rain` values and associated value descriptors
*/
use super::{
    eval::Apply,
    lifetime::{LifetimeBorrow, Live},
    typing::{universe::FINITE_TY, Type, Typed},
};
use crate::value::{
    expr::Sexpr,
    tuple::{Product, Tuple},
    NormalValue, TypeId, TypeRef, UniverseRef, ValId, Value, ValueEnum, VarId,
};
use crate::{debug_from_display, lifetime_region, quick_pretty, trivial_substitute};
use lazy_static::lazy_static;
use std::convert::TryFrom;

pub mod finite;
pub mod logical;

lazy_static! {
    /// An instance of the unit value
    pub static ref UNIT: VarId<()> = VarId::direct_new(());
    /// An instance of the unit type
    pub static ref UNIT_TY: VarId<Unit> = VarId::direct_new(Unit);
}

impl PartialEq<()> for Tuple {
    #[inline]
    fn eq(&self, _: &()) -> bool {
        *self.ty().as_norm() == Unit
    }
}

impl PartialEq<Tuple> for () {
    #[inline]
    fn eq(&self, tuple: &Tuple) -> bool {
        tuple.eq(self)
    }
}

impl PartialEq<()> for Sexpr {
    #[inline]
    fn eq(&self, _: &()) -> bool {
        self.len() == 0
    }
}

impl PartialEq<Sexpr> for () {
    #[inline]
    fn eq(&self, expr: &Sexpr) -> bool {
        expr.eq(self)
    }
}

impl PartialEq<()> for ValueEnum {
    #[inline]
    fn eq(&self, u: &()) -> bool {
        match self {
            //TODO: singletons, or is that a `JEq` only business?
            ValueEnum::Sexpr(s) => s.eq(u),
            ValueEnum::Tuple(t) => t.eq(u),
            _ => false,
        }
    }
}

impl PartialEq<ValueEnum> for () {
    #[inline]
    fn eq(&self, value: &ValueEnum) -> bool {
        value.eq(self)
    }
}

impl PartialEq<NormalValue> for () {
    #[inline]
    fn eq(&self, value: &NormalValue) -> bool {
        value.eq(self)
    }
}

impl From<()> for ValueEnum {
    fn from(_: ()) -> ValueEnum {
        ValueEnum::Sexpr(Sexpr::unit())
    }
}

impl From<()> for NormalValue {
    fn from(_: ()) -> NormalValue {
        NormalValue(ValueEnum::from(()))
    }
}

impl TryFrom<ValueEnum> for () {
    type Error = ValueEnum;
    #[inline]
    fn try_from(value: ValueEnum) -> Result<(), ValueEnum> {
        if value == () {
            Ok(())
        } else {
            Err(value)
        }
    }
}

impl<'a, 'b> TryFrom<&'a ValueEnum> for &'b () {
    type Error = &'a ValueEnum;
    #[inline]
    fn try_from(value: &'a ValueEnum) -> Result<&'b (), &'a ValueEnum> {
        if value == &() {
            Ok(&())
        } else {
            Err(value)
        }
    }
}

impl<'a, 'b> TryFrom<&'a NormalValue> for &'b () {
    type Error = &'a NormalValue;
    #[inline]
    fn try_from(value: &'a NormalValue) -> Result<&'b (), &'a NormalValue> {
        if value == &() {
            Ok(&())
        } else {
            Err(value)
        }
    }
}

impl From<()> for ValId {
    #[inline]
    fn from(_: ()) -> ValId {
        UNIT.as_val().clone()
    }
}

impl Live for () {
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

lifetime_region!(());

impl Typed for () {
    #[inline]
    fn ty(&self) -> TypeRef {
        UNIT_TY.borrow_ty()
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

impl Apply for () {}

impl Value for () {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Attempted to get dependency {} of the unit value, but `()` has no dependencies!",
            ix
        )
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        ().into()
    }
}

trivial_substitute!(());

/**
The unit type

This is a singleton struct representing values of the unit type. It implements efficient conversion to `ValId` and `TypeId`
(as well as `VarId<Unit>`) and is the recommended way to get values of these types corresponding to the unit type.
*/
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Unit;

impl Apply for Unit {}

impl Typed for Unit {
    #[inline]
    fn ty(&self) -> TypeRef {
        unimplemented!()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
    #[inline]
    fn is_kind(&self) -> bool {
        false
    }
}

impl Live for Unit {
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

lifetime_region!(Unit);

impl Type for Unit {
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

quick_pretty!(Unit, "#unit");
debug_from_display!(Unit);

impl PartialEq<Unit> for Product {
    #[inline]
    fn eq(&self, _: &Unit) -> bool {
        self.len() == 0 && !self.is_substruct()
    }
}

impl PartialEq<Product> for Unit {
    #[inline]
    fn eq(&self, product: &Product) -> bool {
        product.eq(self)
    }
}

impl PartialEq<Unit> for ValueEnum {
    fn eq(&self, u: &Unit) -> bool {
        match self {
            //TODO: singletons, or is that a `JEq` only business?
            ValueEnum::Product(p) => p.eq(u),
            _ => false,
        }
    }
}

impl PartialEq<ValueEnum> for Unit {
    fn eq(&self, value: &ValueEnum) -> bool {
        value.eq(self)
    }
}

impl PartialEq<NormalValue> for Unit {
    #[inline]
    fn eq(&self, value: &NormalValue) -> bool {
        value.eq(self)
    }
}

impl From<Unit> for ValueEnum {
    #[inline]
    fn from(_: Unit) -> ValueEnum {
        ValueEnum::Product(Product::unit_ty())
    }
}

impl From<Unit> for NormalValue {
    #[inline]
    fn from(_: Unit) -> NormalValue {
        NormalValue(ValueEnum::from(Unit))
    }
}

impl TryFrom<ValueEnum> for Unit {
    type Error = ValueEnum;
    #[inline]
    fn try_from(value: ValueEnum) -> Result<Unit, ValueEnum> {
        if value == Unit {
            Ok(Unit)
        } else {
            Err(value)
        }
    }
}

impl From<Unit> for ValId {
    #[inline]
    fn from(_: Unit) -> ValId {
        UNIT_TY.as_val().clone()
    }
}

impl From<Unit> for TypeId {
    #[inline]
    fn from(_: Unit) -> TypeId {
        UNIT_TY.as_ty().clone()
    }
}

impl<'a, 'b> TryFrom<&'a ValueEnum> for &'b Unit {
    type Error = &'a ValueEnum;
    #[inline]
    fn try_from(value: &'a ValueEnum) -> Result<&'b Unit, &'a ValueEnum> {
        if value == &Unit {
            Ok(&Unit)
        } else {
            Err(value)
        }
    }
}

impl<'a, 'b> TryFrom<&'a NormalValue> for &'b Unit {
    type Error = &'a NormalValue;
    #[inline]
    fn try_from(value: &'a NormalValue) -> Result<&'b Unit, &'a NormalValue> {
        if value == &Unit {
            Ok(&Unit)
        } else {
            Err(value)
        }
    }
}

impl Value for Unit {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Attempted to get dependency {} of the unit type, but `Unit` has no dependencies!",
            ix
        )
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

trivial_substitute!(Unit);

/// The empty type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Empty;

quick_pretty!(Empty, "#empty");
debug_from_display!(Empty);

#[cfg(test)]
mod tests {
    use super::*;
    /// Construction of the unit value and unit type in various manners yields the correct results
    #[test]
    fn unit_construction() {
        let unit: ValId = ().into();
        let unit_sexpr: ValId = Sexpr::unit().into();
        let unit_tuple: ValId = Tuple::unit().into();
        let unit_cached = UNIT.clone();
        let unit_ty: TypeId = Unit.into();
        let unit_ty_product: TypeId = Product::unit_ty().into();
        let unit_ty_val: ValId = Unit.into();
        let unit_ty_product_val: ValId = Product::unit_ty().into();
        let unit_ty_cached = UNIT_TY.clone();
        assert_eq!(unit, unit_sexpr);
        assert_eq!(unit, unit_tuple);
        assert_eq!(unit, unit_cached);
        assert_eq!(unit_ty, unit_ty_product);
        assert_eq!(unit_ty, unit_ty_val);
        assert_eq!(unit_ty, unit_ty_product_val);
        assert_eq!(unit_ty, unit_ty_cached);
        assert_eq!(unit.ty(), unit_ty);
        assert_ne!(unit_ty, unit);
    }
}
