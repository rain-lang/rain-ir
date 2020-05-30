/*!
Tuples of `rain` values and their associated finite (Cartesian) product types
*/
use super::{
    lifetime::{Lifetime, LifetimeBorrow, Live},
    primitive::UNIT_TY,
    typing::{Type, Typed},
    universe::{Universe, FINITE_TY},
    TypeId, TypeRef, ValId, ValueEnum,
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
    ///
    /// TODO: Optional?
    ty: TypeId,
}

impl Tuple {
    /// Create the tuple corresponding to the element of the unit type
    #[inline]
    pub fn unit() -> Tuple {
        Tuple {
            elems: TupleElems::new(),
            lifetime: Lifetime::default(),
            ty: UNIT_TY.clone(),
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
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
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
    ///
    /// TODO: Optional?
    ty: TypeId,
}

impl Product {
    /// Try to create a new product from a vector of types. Return an error if they have incompatible lifetimes.
    #[inline]
    pub fn new(elems: ProductElems) -> Result<Product, ()> {
        let lifetime = Lifetime::default().intersect(elems.iter().map(|t| t.lifetime()))?.clone_lifetime();
        let ty = Universe::union_all(&Universe::finite(), elems.iter().map(|t| t.universe())).into();
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
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
}

impl Type for Product {
    fn universe(&self) -> Universe {
        match self.ty().as_enum() {
            ValueEnum::Universe(u) => u.clone(),
            _ => unimplemented!(),
        }
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Formatter};

    impl PrettyPrint for Tuple {
        fn prettyprint(
            &self,
            _printer: &mut PrettyPrinter,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }

    impl PrettyPrint for Product {
        fn prettyprint(
            &self,
            _printer: &mut PrettyPrinter,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }
}
