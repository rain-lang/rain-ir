/*!
Boolean types and logical operations
*/

use crate::prettyprinter::tokens::*;
use crate::value::{
    eval::Apply,
    function::pi::Pi,
    lifetime::{LifetimeBorrow, Live, Region, RegionData},
    typing::{Type, Typed},
    universe::FINITE_TY,
    TypeRef, UniverseRef, ValId, Value, VarId,
};
use crate::{debug_from_display, quick_pretty, trivial_substitute, display_pretty};
use lazy_static::lazy_static;
use smallvec::smallvec;
use std::fmt::{self, Display, Formatter};

/// The type of booleans
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Bool;

lazy_static! {
    /// A reference to the type of booleans
    pub static ref BOOL_TY: VarId<Bool> = VarId::direct_new(Bool);
}

debug_from_display!(Bool);
quick_pretty!(Bool, "{}", KEYWORD_BOOL);

impl Typed for Bool {
    #[inline]
    fn ty(&self) -> TypeRef {
        FINITE_TY.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Apply for Bool {}

impl Value for Bool {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!("Bool has no dependencies (asked for dependency #{})", ix)
    }
}

impl Type for Bool {
    #[inline]
    fn is_universe(&self) -> bool {
        false
    }
    #[inline]
    fn universe(&self) -> UniverseRef {
        FINITE_TY.borrow_var()
    }
}

impl Live for Bool {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

impl Typed for bool {
    #[inline]
    fn ty(&self) -> TypeRef {
        BOOL_TY.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
}

impl Apply for bool {}

impl Live for bool {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

impl Value for bool {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Boolean #{} has no dependencies (asked for dependency #{})",
            self, ix
        )
    }
}

trivial_substitute!(bool);
trivial_substitute!(Bool);

lazy_static! {
    /// Regions corresponding to primitive logical operations
    static ref LOGICAL_OP_REGIONS: [Region; 8] = [
        Region::new(RegionData::with(smallvec![Bool.into(); 1], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 2], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 3], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 4], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 5], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 6], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 7], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 8], None)),
    ];
    /// Types corresponding to primitive logical operations
    static ref LOGICAL_OP_TYS: [VarId<Pi>; 8] = [
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[0].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[1].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[2].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[3].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[4].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[5].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[6].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[7].clone()).unwrap().into(),
    ];
}

/// A boolean operation, operating on up to eight booleans
#[derive(Clone)]
pub struct LogicalOperation {
    data: [u8; 256 / 8],
    ty: u8,
}

impl LogicalOperation {
    /// Get the number of bits of this logical operation
    #[inline]
    pub fn no_bits(&self) -> usize {
        1 << self.ty
    }
    /// Get a bit of this logical operation
    #[inline]
    pub fn get_bit(&self, bit: usize) -> bool {
        (self.data[bit / 8] & (1 << bit % 8)) != 0
    }
}

impl Display for LogicalOperation {
    fn fmt(&self, _fmt: &mut Formatter) -> Result<(), fmt::Error> {
        unimplemented!()
    }
}

debug_from_display!(LogicalOperation);
display_pretty!(LogicalOperation);

/// The logical not operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Not;
debug_from_display!(Not);
quick_pretty!(Not, "{}", KEYWORD_NOT);

/// The logical and operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct And;
debug_from_display!(And);
quick_pretty!(And, "{}", KEYWORD_AND);

/// The logical or operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Or;
debug_from_display!(Or);
quick_pretty!(Or, "{}", KEYWORD_OR);

/// The logical xor operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Xor;
debug_from_display!(Xor);
quick_pretty!(Xor, "{}", KEYWORD_XOR);

/// The logical xnor operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Xnor;
debug_from_display!(Xnor);
quick_pretty!(Xnor, "{}", KEYWORD_XNOR);

/// The logical nor operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Nor;
debug_from_display!(Nor);
quick_pretty!(Nor, "{}", KEYWORD_NOR);

/// The logical nand operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Nand;
debug_from_display!(Nand);
quick_pretty!(Nand, "{}", KEYWORD_NAND);

/// The logical equivalence operation, i.e. iff
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Iff;
debug_from_display!(Iff);
quick_pretty!(Iff, "{}", KEYWORD_IFF);

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use crate::prettyprinter::{
        tokens::{KEYWORD_FALSE, KEYWORD_TRUE},
        PrettyPrint, PrettyPrinter,
    };
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for bool {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            match self {
                true => write!(fmt, "{}", KEYWORD_TRUE),
                false => write!(fmt, "{}", KEYWORD_FALSE),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::builder::Builder;
    use crate::value::ValId;
    #[test]
    fn booleans_parse_properly() {
        let mut builder = Builder::<&str>::new();
        let (rest, expr) = builder.parse_expr("#true").expect("#true");
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(true));

        let (rest, expr) = builder.parse_expr("#false").expect("#false");
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(false));

        let (rest, expr) = builder.parse_expr("#bool").expect("#bool");
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(Bool));

        assert!(builder.parse_expr("#fals").is_err());
    }
}
