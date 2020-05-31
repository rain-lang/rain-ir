/*!
Tuples of `rain` values and their associated finite (Cartesian) product types
*/
use super::{
    lifetime::{Lifetime, LifetimeBorrow, Live},
    primitive::UNIT_TY,
    typing::{Type, Typed},
    universe::FINITE_TY,
    TypeId, TypeRef, UniverseId, UniverseRef, ValId, Value,
};
use crate::{debug_from_display, pretty_display};
use smallvec::SmallVec;
use std::ops::Deref;

/// The size of a small tuple
pub const SMALL_TUPLE_SIZE: usize = 3;

/// The size of a small product type
pub const SMALL_PRODUCT_SIZE: usize = SMALL_TUPLE_SIZE;

/// The element-vector of a tuple
pub type TupleElems = SmallVec<[ValId; SMALL_TUPLE_SIZE]>;

/// The element-vector of a product type
pub type ProductElems = SmallVec<[TypeId; SMALL_PRODUCT_SIZE]>;

/// A tuple of `rain` values
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Tuple {
    /// The elements of this tuple
    elems: TupleElems,
    /// The (cached) lifetime of this tuple
    lifetime: Lifetime,
    /// The (cached) type of this tuple
    ty: TypeId,
}

impl Tuple {
    /// Try to create a new product from a vector of values. Return an error if they have incompatible lifetimes.
    #[inline]
    pub fn new(elems: TupleElems) -> Result<Tuple, ()> {
        let lifetime = Lifetime::default()
            .intersect(elems.iter().map(|t| t.lifetime()))?
            .clone_lifetime();
        let ty = Product::new(elems.iter().map(|elem| elem.ty().clone_ty()).collect())?.into();
        Ok(Tuple {
            elems,
            lifetime,
            ty,
        })
    }
    /// Create the tuple corresponding to the element of the unit type
    #[inline]
    pub fn unit() -> Tuple {
        Tuple {
            elems: TupleElems::new(),
            lifetime: Lifetime::default(),
            ty: UNIT_TY.as_ty().clone(),
        }
    }
}

impl Live for Tuple {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

impl Deref for Tuple {
    type Target = TupleElems;
    #[inline]
    fn deref(&self) -> &TupleElems {
        &self.elems
    }
}

impl Typed for Tuple {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
}

impl Value for Tuple {
    #[inline]
    fn no_deps(&self) -> usize {
        self.len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self[ix]
    }
}

debug_from_display!(Tuple);
pretty_display!(Tuple, "[...]");

/// A product of `rain` values
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Product {
    /// The elements of this product type
    elems: ProductElems,
    /// The (cached) lifetime of this product type
    lifetime: Lifetime,
    /// The (cached) type of this product type
    ty: UniverseId,
}

impl Product {
    /// Try to create a new product from a vector of types. Return an error if they have incompatible lifetimes.
    #[inline]
    pub fn new(elems: ProductElems) -> Result<Product, ()> {
        let lifetime = Lifetime::default()
            .intersect(elems.iter().map(|t| t.lifetime()))?
            .clone_lifetime();
        let ty = FINITE_TY.union_all(elems.iter().map(|t| t.universe()));
        Ok(Product {
            elems,
            lifetime,
            ty,
        })
    }
    /// Create the product corresponding to the unit type
    #[inline]
    pub fn unit_ty() -> Product {
        Product {
            elems: SmallVec::new(),
            lifetime: Lifetime::default(),
            ty: FINITE_TY.clone(),
        }
    }
}

debug_from_display!(Product);
pretty_display!(Product, "#product [...]");

impl Live for Product {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

impl Deref for Product {
    type Target = ProductElems;
    #[inline]
    fn deref(&self) -> &ProductElems {
        &self.elems
    }
}

impl Typed for Product {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Type for Product {
    fn universe(&self) -> UniverseRef {
        self.ty.borrow_var()
    }
    fn is_universe(&self) -> bool {
        false
    }
}

impl Value for Product {
    #[inline]
    fn no_deps(&self) -> usize {
        self.len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        (&self[ix]).into()
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use crate::value::primitive::Unit;
    use std::fmt::{self, Formatter, Display};

    impl PrettyPrint for Tuple {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            if self.len() == 0 {
                return write!(fmt, "()");
            }
            write!(fmt, "[")?;
            let mut first = true;
            for elem in self.iter() {
                if !first { write!(fmt, " ")?; }
                first = false;
                elem.prettyprint(printer, fmt)?;
            }
            write!(fmt, "]")
        }
    }

    impl PrettyPrint for Product {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            if self.len() == 0 {
                return write!(fmt, "{}", Unit);
            }
            write!(fmt, "#product[")?;
            let mut first = true;
            for elem in self.iter() {
                if !first { write!(fmt, " ")?; }
                first = false;
                elem.prettyprint(printer, fmt)?;
            }
            write!(fmt, "]")
        }
    }

    #[cfg(test)]
    mod tests {
        use smallvec::smallvec;
        use super::*;
        #[test]
        fn nested_units_print_properly() {
            let unit = Tuple::unit();
            let unit_ty = Product::unit_ty();
            assert_eq!(format!("{}", unit), "()");
            assert_eq!(format!("{}", unit_ty), format!("{}", Unit));
            let two_units = Tuple::new(smallvec![unit.clone().into(), unit.into()])
                .expect("This is a valid tuple!");
            assert_eq!(format!("{}", two_units), "[() ()]");
            let unit_squared = two_units.ty();
            assert_eq!(format!("{}", unit_squared), format!("#product[{} {}]", Unit, Unit));
        }
    }
}
