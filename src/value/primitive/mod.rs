/*!
Primitive `rain` values and associated value descriptors
*/
use super::{
    expr::Sexpr,
    lifetime::{LifetimeBorrow, Live},
    tuple::{Product, Tuple},
    typing::{Type, Typed},
    universe::FINITE_TY,
    NormalValue, TypeId, TypeRef, UniverseRef, ValId, ValueEnum,
};
use crate::{debug_from_display, quick_pretty};
use lazy_static::lazy_static;
use std::convert::TryFrom;

lazy_static! {
    /// An instance of the unit value
    pub static ref UNIT: ValId = ValId::from(ValueEnum::from(()));
    /// An instance of the unit type
    pub static ref UNIT_TY: TypeId = TypeId::assert_normal_ty(Unit);
}

impl PartialEq<()> for Tuple {
    #[inline]
    fn eq(&self, _: &()) -> bool {
        self.len() == 0
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

impl PartialEq<()> for NormalValue {
    #[inline]
    fn eq(&self, u: &()) -> bool {
        self.0.eq(u)
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
        UNIT.clone()
    }
}

impl Live for () {
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

impl Typed for () {
    fn ty(&self) -> TypeRef {
        UNIT_TY.borrow_ty()
    }
}

/// The unit type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Unit;

impl Typed for Unit {
    fn ty(&self) -> TypeRef {
        unimplemented!()
    }
}

impl Live for Unit {
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

impl Type for Unit {
    fn universe(&self) -> UniverseRef {
        FINITE_TY.borrow_var()
    }
}

quick_pretty!(Unit, "#unit");
debug_from_display!(Unit);

impl PartialEq<Unit> for Product {
    #[inline]
    fn eq(&self, _: &Unit) -> bool {
        self.len() == 0
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

impl PartialEq<Unit> for NormalValue {
    #[inline]
    fn eq(&self, u: &Unit) -> bool {
        self.0.eq(u)
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
        UNIT_TY.clone()
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

/// The empty type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Empty;

quick_pretty!(Empty, "#empty");
debug_from_display!(Empty);
