/*!
Primitive `rain` values and associated value descriptors
*/
use super::{
    expr::Sexpr,
    tuple::{Product, Tuple},
    ValueEnum,
};
use crate::{debug_from_display, quick_pretty};

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

impl PartialEq<ValueEnum> for () {
    fn eq(&self, value: &ValueEnum) -> bool {
        match value {
            //TODO: singletons, or is that a `JEq` only business?
            ValueEnum::Sexpr(s) => self.eq(s),
            ValueEnum::Tuple(t) => self.eq(t),
            _ => false,
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

impl PartialEq<ValueEnum> for Unit {
    fn eq(&self, value: &ValueEnum) -> bool {
        match value {
            //TODO: singletons, or is that a `JEq` only business?
            ValueEnum::Product(p) => self.eq(p),
            _ => false,
        }
    }
}

/// The empty type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Empty;

quick_pretty!(Empty, "#empty");
debug_from_display!(Empty);
