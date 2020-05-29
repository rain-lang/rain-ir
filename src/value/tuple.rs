/*!
Tuples of `rain` values and their associated finite (Cartesian) product types
*/
use crate::{debug_from_display, display_pretty};
use super::{ValId, TypeId, region::Region}

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
    /// The (cached) region of this tuple
    region: Option<Region>,
    /// The (cached) type of this tuple
    ///
    /// TODO: Optional?
    ty: TypeId,
}

/// A product of `rain` values
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Product {
    /// The elements of this product type
    elems: TupleElems,
    /// The (cached) region of this product type
    region: Option<Region>,
    /// The (cached) type of this product type
    ///
    /// TODO: Optional?
    ty: TypeId,
}