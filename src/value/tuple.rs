/*!
Tuples of `rain` values and their associated finite (Cartesian) product types
*/
use super::{
    lifetime::{Lifetime, LifetimeBorrow, Live},
    universe::Universe,
    TypeId, ValId,
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
            ty: Product::unit_ty().into()
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
    /// Create the product corresponding to the unit type
    #[inline]
    pub fn unit_ty() -> Product {
        Product {
            elems: SmallVec::new(),
            lifetime: Lifetime::default(),
            ty: Universe::finite().into(),
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
