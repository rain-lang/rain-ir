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
use crate::{debug_from_display, display_pretty, quick_pretty, trivial_substitute};
use either::Either;
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
    static ref LOGICAL_OP_REGIONS: [Region; 3] = [
        Region::new(RegionData::with(smallvec![Bool.into(); 1], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 2], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 3], None)),
    ];
    /// Types corresponding to primitive logical operations
    static ref LOGICAL_OP_TYS: [VarId<Pi>; 3] = [
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[0].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[1].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[2].clone()).unwrap().into(),
    ];
}

/// Masks corresponding to what bits must be set for operations of a given arity
pub const LOGICAL_OP_ARITY_MASKS: [u8; 4] = [
    0b1,        // Nullary
    0b11,       // Unary
    0b1111,     // Binary
    0b11111111, // Ternary
];

/// A boolean operation, operating on up to three
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LogicalOperation {
    /// The data backing this logical operation
    data: u8,
    /// The arity of this logical operation
    arity: u8,
}

impl LogicalOperation {
    /// Create a new logical operation with a given type and data set.
    /// Return an error if the arity is zero, or greater than three, or
    /// if there are nonzero bits corresponding to higher arities
    #[inline]
    pub fn try_new(arity: u8, data: u8) -> Result<LogicalOperation, ()> {
        if arity == 0 || arity > 7 || !LOGICAL_OP_ARITY_MASKS[arity as usize] & data != 0 {
            Err(())
        } else {
            Ok(LogicalOperation { arity, data })
        }
    }
    /// Create a constant logical operation with a given arity.
    /// Return an error if the arity is zero, or greater than three
    #[inline]
    pub fn try_const(arity: u8, value: bool) -> Result<LogicalOperation, ()> {
        if arity == 0 || arity > 3 {
            Err(())
        } else {
            Ok(LogicalOperation {
                arity,
                data: if value {
                    LOGICAL_OP_ARITY_MASKS[arity as usize]
                } else {
                    0
                },
            })
        }
    }
    /// Create a new unary logical operation
    #[inline]
    pub fn unary(low: bool, high: bool) -> LogicalOperation {
        let low = low as u8;
        let high = (high as u8) << 1;
        Self::try_new(1, low | high).expect("Unary operations are valid")
    }
    /// Create a new binary logical operation.
    #[inline]
    pub fn binary(ff: bool, ft: bool, tf: bool, tt: bool) -> LogicalOperation {
        let data = ff as u8 + ((ft as u8) << 1) + ((tf as u8) << 2) + ((tt as u8) << 3);
        Self::try_new(2, data).expect("Binary operations are valid")
    }
    /// Create a new ternary logical operation
    #[inline]
    pub fn ternary(data: u8) -> LogicalOperation {
        Self::try_new(3, data).expect("Ternary operations are valid")
    }
    /// Get the number of bits of this logical operation
    #[inline]
    pub fn no_bits(&self) -> usize {
        1 << self.arity
    }
    /// Get a bit of this logical operation. Can also be viewed as completely evaluating it.
    #[inline]
    pub fn get_bit(&self, bit: u8) -> bool {
        self.data & (1 << bit) != 0
    }
    /// Evaluate a logical operation, getting either a result or a partial evaluation
    #[inline]
    pub fn apply(&self, value: bool) -> Either<bool, LogicalOperation> {
        if self.arity == 1 {
            Either::Left(self.get_bit(value as u8))
        } else {
            let arity = self.arity - 1;
            let shift = if value { 1 << arity } else { 0 };
            let mask = LOGICAL_OP_ARITY_MASKS[arity as usize] << shift;
            Either::Right(LogicalOperation {
                arity: arity,
                data: (self.data & mask) >> shift,
            })
        }
    }
}

impl Display for LogicalOperation {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self.arity {
            1 => write!(
                fmt,
                "{}({}, {:#04b})",
                KEYWORD_LOGICAL, self.arity, self.data
            ),
            2 => write!(
                fmt,
                "{}({}, {:#06b})",
                KEYWORD_LOGICAL, self.arity, self.data
            ),
            _ => write!(
                fmt,
                "{}({}, {:#010b})",
                KEYWORD_LOGICAL, self.arity, self.data
            ),
        }
    }
}

debug_from_display!(LogicalOperation);
display_pretty!(LogicalOperation);

macro_rules! make_logical {
    ($t:ty[$arity:expr] = $tt:expr) => {
        impl From<$t> for LogicalOperation {
            #[inline]
            fn from(_: $t) -> LogicalOperation {
                LogicalOperation::try_new($arity, $tt).unwrap()
            }
        }
        impl PartialEq<LogicalOperation> for $t {
            #[inline]
            fn eq(&self, l: &LogicalOperation) -> bool {
                LogicalOperation::from(*self).eq(l)
            }
        }
        impl PartialEq<$t> for LogicalOperation {
            #[inline]
            fn eq(&self, t: &$t) -> bool {
                LogicalOperation::from(*t).eq(self)
            }
        }
    };
    ($t:ty = $tt:expr) => {
        make_logical!($t[2] = $tt);
    };
}

/// The logical identity operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Id;
debug_from_display!(Id);
quick_pretty!(Id, "{}", KEYWORD_LOGICAL_ID);
make_logical!(Id[1] = 0b10);

/// The logical not operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Not;
debug_from_display!(Not);
quick_pretty!(Not, "{}", KEYWORD_NOT);
make_logical!(Not[1] = 0b01);

/// The logical and operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct And;
debug_from_display!(And);
quick_pretty!(And, "{}", KEYWORD_AND);
make_logical!(And = 0b1000);

/// The logical or operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Or;
debug_from_display!(Or);
quick_pretty!(Or, "{}", KEYWORD_OR);
make_logical!(Or = 0b1110);

/// The logical xor operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Xor;
debug_from_display!(Xor);
quick_pretty!(Xor, "{}", KEYWORD_XOR);
make_logical!(Xor = 0b0110);

/// The logical nor operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Nor;
debug_from_display!(Nor);
quick_pretty!(Nor, "{}", KEYWORD_NOR);
make_logical!(Nor = 0b0001);

/// The logical nand operation
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Nand;
debug_from_display!(Nand);
quick_pretty!(Nand, "{}", KEYWORD_NAND);
make_logical!(Nand = 0b0111);

/// The logical equivalence operation, i.e. iff/xnor/nxor
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Iff;
debug_from_display!(Iff);
quick_pretty!(Iff, "{}", KEYWORD_IFF);
make_logical!(Iff = 0b1001);

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

    #[test]
    fn logical_operations_sanity_check() {
        // Sanity checks: (in)equality
        assert_ne!(LogicalOperation::from(And), LogicalOperation::from(Or));
        assert_ne!(LogicalOperation::from(And), LogicalOperation::from(Not));
        assert_ne!(LogicalOperation::from(Not), LogicalOperation::from(Id));

        // Sanity checks: construction
        assert_eq!(
            LogicalOperation::unary(false, true),
            LogicalOperation::from(Id)
        );
        assert_eq!(
            LogicalOperation::binary(false, true, true, false),
            LogicalOperation::from(Xor)
        );

        // Application works
        assert_eq!(
            LogicalOperation::from(And).apply(true),
            Either::Right(LogicalOperation::from(Id)),
        );
        assert_eq!(
            LogicalOperation::from(And).apply(false),
            Either::Right(LogicalOperation::try_const(1, false).unwrap())
        );
        assert_eq!(
            LogicalOperation::from(And)
                .apply(true)
                .right()
                .unwrap()
                .apply(false),
            Either::Left(false)
        );
        assert_eq!(
            LogicalOperation::from(And)
                .apply(true)
                .right()
                .unwrap()
                .apply(true),
            Either::Left(true)
        );
    }

    /// Test a binary operation exhaustively
    fn test_binary_operation(
        op: LogicalOperation,
        partial_table: &[LogicalOperation; 2],
        truth_table: &[bool; 4],
    ) {
        for left in [true, false].iter().copied() {
            for right in [true, false].iter().copied() {
                let ix = left as u8 | (right as u8) << 1;
                assert_eq!(op.get_bit(ix), truth_table[ix as usize]);
                let partial = op
                    .apply(left)
                    .right()
                    .expect("Expected binary operation, got unary!");
                assert_eq!(
                    partial, partial_table[left as usize],
                    "Incorrect partial evaluation of ({} {})",
                    op, left
                );
                let fin = partial
                    .apply(right)
                    .left()
                    .expect("Expected binary operation, got arity > 2!");
                assert_eq!(
                    fin, truth_table[ix as usize],
                    "Incorrect total evaluation of ({} {} {}) == ({} {})",
                    op, left, right, partial, right
                );
            }
        }
    }

    fn cl(b: bool) -> LogicalOperation {
        LogicalOperation::try_const(1, b).unwrap()
    }

    #[test]
    fn test_binary_operations() {
        let binary_ops: &[(LogicalOperation, [LogicalOperation; 2], [bool; 4])] = &[
            (
                And.into(),
                [cl(false), Id.into()],
                [false, false, false, true],
            ),
            (Or.into(), [Id.into(), cl(true)], [false, true, true, true]),
            (
                Xor.into(),
                [Id.into(), Not.into()],
                [false, true, true, false],
            ),
            (
                Nor.into(),
                [Not.into(), cl(false)],
                [true, false, false, false],
            ),
            (
                Nand.into(),
                [cl(true), Not.into()],
                [true, true, true, false],
            ),
            (
                Iff.into(),
                [Not.into(), Id.into()],
                [true, false, false, true],
            ),
        ];
        for (op, partial_table, truth_table) in binary_ops.iter() {
            test_binary_operation(*op, partial_table, truth_table);
        }
    }
}
