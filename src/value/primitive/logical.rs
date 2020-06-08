/*!
Boolean types and logical operations
*/

use crate::prettyprinter::tokens::*;
use crate::value::{
    eval::{Application, Apply, EvalCtx},
    function::pi::Pi,
    lifetime::{LifetimeBorrow, Live, Region, RegionData},
    typing::{Type, Typed},
    universe::FINITE_TY,
    Error, NormalValue, TypeId, TypeRef, UniverseRef, ValId, Value, ValueEnum, VarId,
};
use crate::{debug_from_display, display_pretty, normal_valid, quick_pretty, trivial_substitute};
use either::Either;
use lazy_static::lazy_static;
use ref_cast::RefCast;
use smallvec::smallvec;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

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
    static ref LOGICAL_OP_REGIONS: [Region; 7] = [
        Region::new(RegionData::with(smallvec![Bool.into(); 1], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 2], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 3], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 4], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 5], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 6], None)),
        Region::new(RegionData::with(smallvec![Bool.into(); 7], None)),
    ];
    /// Types corresponding to primitive logical operations
    static ref LOGICAL_OP_TYS: [VarId<Pi>; 7] = [
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[0].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[1].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[2].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[3].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[4].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[5].clone()).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[6].clone()).unwrap().into(),
    ];
}

/// Masks corresponding to what bits must be set for operations of a given arity
pub const LOGICAL_OP_ARITY_MASKS: [u128; 8] = [
    0b1,                                // Nullary
    0b11,                               // Unary
    0xF,                                // Binary
    0xFF,                               // Ternary
    0xFFFF,                             // Arity 4
    0xFFFFFFFF,                         // Arity 5,
    0xFFFFFFFFFFFFFFFF,                 // Arity 6
    0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF, // Arity 7
];

/// A boolean operation, operating on up to seven booleans
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Logical {
    /// The data backing this logical operation
    data: u128,
    /// The arity of this logical operation
    arity: u8,
}

impl Logical {
    /// Create a new logical operation with a given type and data set.
    /// Return an error if the arity is zero, or greater than seven, or
    /// if there are nonzero bits corresponding to higher arities
    #[inline]
    pub fn try_new(arity: u8, data: u128) -> Result<Logical, ()> {
        if arity == 0 || arity > 7 || !LOGICAL_OP_ARITY_MASKS[arity as usize] & data != 0 {
            Err(())
        } else {
            Ok(Logical { arity, data })
        }
    }
    /// Create a constant logical operation with a given arity.
    /// Return an error if the arity is zero, or greater than seven
    #[inline]
    pub fn try_const(arity: u8, value: bool) -> Result<Logical, ()> {
        if arity == 0 || arity > 7 {
            Err(())
        } else {
            Ok(Logical {
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
    pub fn unary(low: bool, high: bool) -> Logical {
        let low = low as u128;
        let high = (high as u128) << 1;
        Self::try_new(1, low | high).expect("Unary operations are valid")
    }
    /// Create a new binary logical operation.
    #[inline]
    pub fn binary(ff: bool, ft: bool, tf: bool, tt: bool) -> Logical {
        let data = ff as u128 + ((ft as u128) << 1) + ((tf as u128) << 2) + ((tt as u128) << 3);
        Self::try_new(2, data).expect("Binary operations are valid")
    }
    /// Create a new ternary logical operation
    #[inline]
    pub fn ternary(data: u8) -> Logical {
        Self::try_new(3, data as u128).expect("Ternary operations are valid")
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
    pub fn apply(&self, value: bool) -> Either<bool, Logical> {
        if self.arity == 1 {
            Either::Left(self.get_bit(value as u8))
        } else {
            let arity = self.arity - 1;
            let shift = if value { 1 << arity } else { 0 };
            let mask = LOGICAL_OP_ARITY_MASKS[arity as usize] << shift;
            Either::Right(Logical {
                arity: arity,
                data: (self.data & mask) >> shift,
            })
        }
    }
    /// Print this as a raw logical operation
    #[inline]
    pub fn print_raw(&self) -> &RawLogical {
        RefCast::ref_cast(self)
    }
}

impl Display for Logical {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        if *self == Id {
            return write!(fmt, "{}", Id);
        }
        if *self == Not {
            return write!(fmt, "{}", Not);
        }
        if *self == And {
            return write!(fmt, "{}", And);
        }
        if *self == Or {
            return write!(fmt, "{}", Or);
        }
        if *self == Xor {
            return write!(fmt, "{}", Xor);
        }
        if *self == Nor {
            return write!(fmt, "{}", Nor);
        }
        if *self == Nand {
            return write!(fmt, "{}", Nand);
        }
        if *self == Iff {
            return write!(fmt, "{}", Iff);
        }
        write!(fmt, "{}", self.print_raw())
    }
}

/// Format a logical operation with `print_raw`
#[derive(Copy, Clone, PartialEq, Eq, Hash, RefCast)]
#[repr(transparent)]
pub struct RawLogical(pub Logical);

impl Deref for RawLogical {
    type Target = Logical;
    #[inline]
    fn deref(&self) -> &Logical {
        &self.0
    }
}

impl Display for RawLogical {
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
            3 => write!(
                fmt,
                "{}({}, {:#010b})",
                KEYWORD_LOGICAL, self.arity, self.data
            ),
            _ => write!(fmt, "{}({}, {:#x})", KEYWORD_LOGICAL, self.arity, self.data),
        }
    }
}

debug_from_display!(RawLogical);

impl Typed for Logical {
    #[inline]
    fn ty(&self) -> TypeRef {
        LOGICAL_OP_TYS[self.arity as usize - 1].borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Apply for Logical {
    fn do_apply_in_ctx<'a>(
        &self,
        args: &'a [ValId],
        _inline: bool,
        _ctx: Option<&mut EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        // Null evaluation
        if args.len() == 0 {
            return Ok(Application::Stop(
                self.lifetime().clone_lifetime(),
                self.ty().clone_ty(),
            ));
        }
        // Over-evaluation
        if args.len() > self.arity as usize {
            return Err(Error::TooManyArgs);
        }
        let mut l = *self;
        let mut cut_ix = 0;
        // Evaluate
        for (i, arg) in args.iter().enumerate() {
            if arg.ty() != TypeId::from(Bool) {
                return Err(Error::TypeMismatch);
            }
            let ap = if cut_ix == i {
                match arg.as_enum() {
                    ValueEnum::Bool(b) => Some(l.apply(*b)),
                    _ => {
                        let l_t = l.apply(true);
                        let l_f = l.apply(false);
                        if l_t == l_f {
                            Some(l_t)
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            };
            if let Some(ap) = ap {
                cut_ix += 1;
                match ap {
                    Either::Left(b) => return Ok(Application::Success(&args[cut_ix..], b.into())),
                    Either::Right(f) => l = f,
                }
            }
        }
        return Ok(Application::Success(&args[cut_ix..], l.into()));
    }
}

trivial_substitute!(Logical);

impl Value for Logical {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Logical operation {} has no dependencies (asked for dependency #{})",
            self, ix
        )
    }
}

impl Live for Logical {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

debug_from_display!(Logical);
display_pretty!(Logical);

macro_rules! make_logical {
    ($t:ident[$arity:expr] = $tt:expr) => {
        impl From<$t> for Logical {
            #[inline]
            fn from(_: $t) -> Logical {
                Logical::try_new($arity, $tt).unwrap()
            }
        }
        impl PartialEq<Logical> for $t {
            #[inline]
            fn eq(&self, l: &Logical) -> bool {
                Logical::from(*self).eq(l)
            }
        }
        impl PartialEq<$t> for Logical {
            #[inline]
            fn eq(&self, t: &$t) -> bool {
                Logical::from(*t).eq(self)
            }
        }
        impl Typed for $t {
            #[inline]
            fn ty(&self) -> TypeRef {
                LOGICAL_OP_TYS[$arity - 1].borrow_ty()
            }
            #[inline]
            fn is_ty(&self) -> bool {
                false
            }
        }
        impl Live for $t {
            #[inline]
            fn lifetime(&self) -> LifetimeBorrow {
                LifetimeBorrow::default()
            }
        }
        trivial_substitute!($t);
        normal_valid!($t);
        impl Apply for $t {
            fn do_apply_in_ctx<'a>(
                &self,
                args: &'a [ValId],
                inline: bool,
                ctx: Option<&mut EvalCtx>,
            ) -> Result<Application<'a>, Error> {
                Logical::from(*self).do_apply_in_ctx(args, inline, ctx)
            }
        }
        impl Value for $t {
            fn no_deps(&self) -> usize {
                0
            }
            fn get_dep(&self, ix: usize) -> &ValId {
                panic!(
                    "Logical operation {} has no dependencies (tried to get dep #{})",
                    self, ix
                )
            }
        }
        impl From<$t> for ValueEnum {
            fn from(t: $t) -> ValueEnum {
                Logical::from(t).into()
            }
        }
        impl From<$t> for NormalValue {
            fn from(t: $t) -> NormalValue {
                Logical::from(t).into()
            }
        }
        impl TryFrom<Logical> for $t {
            type Error = Logical;
            fn try_from(l: Logical) -> Result<$t, Logical> {
                if l == $t {
                    Ok($t)
                } else {
                    Err(l)
                }
            }
        }
        impl TryFrom<ValueEnum> for $t {
            type Error = ValueEnum;
            fn try_from(v: ValueEnum) -> Result<$t, ValueEnum> {
                let l = Logical::try_from(v)?;
                Ok($t::try_from(l)?)
            }
        }
        impl TryFrom<NormalValue> for $t {
            type Error = NormalValue;
            fn try_from(v: NormalValue) -> Result<$t, NormalValue> {
                let l = Logical::try_from(v)?;
                Ok($t::try_from(l)?)
            }
        }
        impl<'a, 'b> TryFrom<&'a Logical> for &'b $t {
            type Error = &'a Logical;
            fn try_from(l: &'a Logical) -> Result<&'b $t, &'a Logical> {
                if l == &$t {
                    Ok(&$t)
                } else {
                    Err(l)
                }
            }
        }
        impl<'a, 'b> TryFrom<&'a ValueEnum> for &'b $t {
            type Error = &'a ValueEnum;
            fn try_from(v: &'a ValueEnum) -> Result<&'b $t, &'a ValueEnum> {
                let l: &'a Logical = v.try_into()?;
                if l == &$t {
                    Ok(&$t)
                } else {
                    Err(v)
                }
            }
        }
        impl<'a, 'b> TryFrom<&'a NormalValue> for &'b $t {
            type Error = &'a NormalValue;
            fn try_from(v: &'a NormalValue) -> Result<&'b $t, &'a NormalValue> {
                let l: &'a Logical = v.try_into()?;
                if l == &$t {
                    Ok(&$t)
                } else {
                    Err(v)
                }
            }
        }
    };
    ($t:ident = $tt:expr) => {
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
    use crate::prettyprinter::PrettyPrint;
    use crate::value::ValId;
    #[test]
    fn booleans_parse_properly() {
        let mut builder = Builder::<&str>::new();
        let f_true = format!("{}", true.prp());
        let f_false = format!("{}", false.prp());
        let f_bool = format!("{}", Bool);
        let f_bool_prp = format!("{}", Bool.prp());

        let (rest, expr) = builder.parse_expr(&f_true).expect(KEYWORD_TRUE);
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(true));
        let (rest, expr) = builder.parse_expr(&KEYWORD_TRUE).expect(KEYWORD_TRUE);
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(true));

        let (rest, expr) = builder.parse_expr(&f_false).expect(KEYWORD_FALSE);
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(false));
        let (rest, expr) = builder.parse_expr(&KEYWORD_FALSE).expect(KEYWORD_FALSE);
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(false));

        let (rest, expr) = builder.parse_expr(&f_bool).expect(KEYWORD_BOOL);
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(Bool));
        let (rest, expr) = builder.parse_expr(&f_bool_prp).expect(KEYWORD_BOOL);
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(Bool));
        let (rest, expr) = builder.parse_expr(&KEYWORD_BOOL).expect(KEYWORD_BOOL);
        assert_eq!(rest, "");
        assert_eq!(expr, ValId::from(Bool));

        assert!(builder.parse_expr("#fals").is_err());
    }

    #[test]
    fn logical_operations_sanity_check() {
        // Sanity checks: (in)equality
        assert_ne!(Logical::from(And), Logical::from(Or));
        assert_ne!(Logical::from(And), Logical::from(Not));
        assert_ne!(Logical::from(Not), Logical::from(Id));

        // Sanity checks: construction
        assert_eq!(Logical::unary(false, true), Logical::from(Id));
        assert_eq!(
            Logical::binary(false, true, true, false),
            Logical::from(Xor)
        );

        // Application works
        assert_eq!(
            Logical::from(And).apply(true),
            Either::Right(Logical::from(Id)),
        );
        assert_eq!(
            Logical::from(And).apply(false),
            Either::Right(Logical::try_const(1, false).unwrap())
        );
        assert_eq!(
            Logical::from(And).apply(true).right().unwrap().apply(false),
            Either::Left(false)
        );
        assert_eq!(
            Logical::from(And).apply(true).right().unwrap().apply(true),
            Either::Left(true)
        );
    }

    /// Test a binary operation exhaustively
    fn test_binary_operation(
        op: Logical,
        partial_table: &[Logical; 2],
        truth_table: &[bool; 4],
        builder: &mut Builder<String>,
    ) {
        // Test parsing:
        assert_eq!(
            builder.parse_expr(&format!("{}", op)).unwrap(),
            ("", op.into())
        );
        assert_eq!(
            builder.parse_expr(&format!("{}", op.print_raw())).unwrap(),
            ("", op.into())
        );

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
                assert_eq!(
                    builder.parse_expr(&format!("{} #{}", op, left)).unwrap(),
                    ("", partial.into())
                );
                assert_eq!(
                    builder.parse_expr(&format!("{}", partial)).unwrap(),
                    ("", partial.into())
                );
                assert_eq!(
                    builder
                        .parse_expr(&format!("{}", partial.print_raw()))
                        .unwrap(),
                    ("", partial.into())
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
                assert_eq!(
                    builder
                        .parse_expr(&format!("{} #{} #{}", op, left, right))
                        .unwrap(),
                    ("", fin.into())
                );
                assert_eq!(
                    builder
                        .parse_expr(&format!("({} #{}) #{}", op, left, right))
                        .unwrap(),
                    ("", fin.into())
                );
                assert_eq!(
                    builder
                        .parse_expr(&format!("{} #{}", partial, right))
                        .unwrap(),
                    ("", fin.into())
                );
            }
        }
    }

    fn cl(b: bool) -> Logical {
        Logical::try_const(1, b).unwrap()
    }

    #[test]
    fn test_binary_operations() {
        let mut builder = Builder::<String>::new();
        let binary_ops: &[(Logical, [Logical; 2], [bool; 4])] = &[
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
            test_binary_operation(*op, partial_table, truth_table, &mut builder);
        }
    }
}
