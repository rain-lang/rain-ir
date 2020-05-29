/*!
Primitive `rain` values and associated value descriptors
*/
use super::{
    expr::Sexpr,
    tuple::{Product, Tuple},
    ValueEnum,
};
use crate::{debug_from_display, quick_pretty};
use std::convert::TryFrom;

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

/// The unit type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Unit;

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

impl From<Unit> for ValueEnum {
    #[inline]
    fn from(_: Unit) -> ValueEnum { ValueEnum::Product(Product::unit_ty()) }
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

/// The empty type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Empty;

quick_pretty!(Empty, "#empty");
debug_from_display!(Empty);
