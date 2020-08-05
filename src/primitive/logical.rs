/*!
Boolean types and logical operations
*/
use crate::eval::{Application, Apply, EvalCtx};
use crate::function::pi::Pi;
use crate::lifetime::{Lifetime, Live};
use crate::region::Region;
use crate::tokens::*;
use crate::typing::{Type, Typed};
use crate::value::{
    universe::FINITE_TY, Error, NormalValue, TypeId, TypeRef, UniverseRef, ValId, Value, ValueData,
    ValueEnum, VarId,
};
use crate::{
    debug_from_display, display_pretty, enum_convert, normal_valid, quick_pretty, trivial_lifetime,
    trivial_substitute, vararr,
};
use either::Either;
use lazy_static::lazy_static;
use ref_cast::RefCast;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};
use std::ops::{BitAnd, BitOr, BitXor, Deref, Index, Not as NotOp};

/// The type of booleans
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Bool;

lazy_static! {
    /// A reference to the type of booleans
    pub static ref BOOL_TY: VarId<Bool> = VarId::direct_new(Bool);
    /// A reference to the true boolean value
    pub static ref TRUE: VarId<bool> = VarId::direct_new(true);
    /// A reference to the false boolean value
    pub static ref FALSE: VarId<bool> = VarId::direct_new(false);
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
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::BoolTy(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
    #[inline]
    fn into_val(self) -> ValId {
        BOOL_TY.clone_val()
    }
}

impl ValueData for Bool {}

impl Type for Bool {
    #[inline]
    fn is_universe(&self) -> bool {
        false
    }
    #[inline]
    fn universe(&self) -> UniverseRef {
        FINITE_TY.borrow_var()
    }
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

trivial_lifetime!(Bool);

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

trivial_lifetime!(bool);

impl From<Bool> for ValueEnum {
    fn from(b: Bool) -> ValueEnum {
        ValueEnum::BoolTy(b)
    }
}

impl TryFrom<ValueEnum> for Bool {
    type Error = ValueEnum;
    fn try_from(val: ValueEnum) -> Result<Bool, ValueEnum> {
        match val {
            ValueEnum::BoolTy(b) => Ok(b),
            v => Err(v),
        }
    }
}

impl<'a> TryFrom<&'a ValueEnum> for &'a Bool {
    type Error = &'a ValueEnum;
    fn try_from(val: &'a ValueEnum) -> Result<&'a Bool, &'a ValueEnum> {
        match val {
            ValueEnum::BoolTy(b) => Ok(b),
            v => Err(v),
        }
    }
}

impl From<Bool> for NormalValue {
    fn from(b: Bool) -> NormalValue {
        NormalValue(ValueEnum::BoolTy(b))
    }
}

impl TryFrom<NormalValue> for Bool {
    type Error = NormalValue;
    fn try_from(val: NormalValue) -> Result<Bool, NormalValue> {
        match val.deref() {
            ValueEnum::BoolTy(b) => Ok(*b),
            _ => Err(val),
        }
    }
}

impl<'a> TryFrom<&'a NormalValue> for &'a Bool {
    type Error = &'a NormalValue;
    fn try_from(val: &'a NormalValue) -> Result<&'a Bool, &'a NormalValue> {
        match val.deref() {
            ValueEnum::BoolTy(b) => Ok(b),
            _ => Err(val),
        }
    }
}

impl From<bool> for ValueEnum {
    fn from(b: bool) -> ValueEnum {
        ValueEnum::Bool(b)
    }
}

impl TryFrom<ValueEnum> for bool {
    type Error = ValueEnum;
    fn try_from(val: ValueEnum) -> Result<bool, ValueEnum> {
        match val {
            ValueEnum::Bool(b) => Ok(b),
            v => Err(v),
        }
    }
}

impl<'a> TryFrom<&'a ValueEnum> for &'a bool {
    type Error = &'a ValueEnum;
    fn try_from(val: &'a ValueEnum) -> Result<&'a bool, &'a ValueEnum> {
        match val {
            ValueEnum::Bool(b) => Ok(b),
            v => Err(v),
        }
    }
}

impl From<bool> for NormalValue {
    fn from(b: bool) -> NormalValue {
        NormalValue(ValueEnum::Bool(b))
    }
}

impl TryFrom<NormalValue> for bool {
    type Error = NormalValue;
    fn try_from(val: NormalValue) -> Result<bool, NormalValue> {
        match val.deref() {
            ValueEnum::Bool(b) => Ok(*b),
            _ => Err(val),
        }
    }
}

impl<'a> TryFrom<&'a NormalValue> for &'a bool {
    type Error = &'a NormalValue;
    fn try_from(val: &'a NormalValue) -> Result<&'a bool, &'a NormalValue> {
        match val.deref() {
            ValueEnum::Bool(b) => Ok(b),
            _ => Err(val),
        }
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
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Bool(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
    #[inline]
    fn into_val(self) -> ValId {
        if self {
            TRUE.clone_val()
        } else {
            FALSE.clone_val()
        }
    }
}

impl ValueData for bool {}

trivial_substitute!(bool);
trivial_substitute!(Bool);

lazy_static! {
    /// Regions corresponding to primitive logical operations
    pub static ref LOGICAL_OP_REGIONS: [Region; 7] = [
        Region::with(vararr![Bool.into(); 1], None),
        Region::with(vararr![Bool.into(); 2], None),
        Region::with(vararr![Bool.into(); 3], None),
        Region::with(vararr![Bool.into(); 4], None),
        Region::with(vararr![Bool.into(); 5], None),
        Region::with(vararr![Bool.into(); 6], None),
        Region::with(vararr![Bool.into(); 7], None),
    ];
    /// Types corresponding to primitive logical operations
    pub static ref LOGICAL_OP_TYS: [VarId<Pi>; 7] = [
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[0].clone(), Lifetime::STATIC).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[1].clone(), Lifetime::STATIC).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[2].clone(), Lifetime::STATIC).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[3].clone(), Lifetime::STATIC).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[4].clone(), Lifetime::STATIC).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[5].clone(), Lifetime::STATIC).unwrap().into(),
        Pi::try_new(Bool.into(), LOGICAL_OP_REGIONS[6].clone(), Lifetime::STATIC).unwrap().into(),
    ];
}

/// The region of a function taking in a single boolean
pub fn unary_region() -> Region {
    LOGICAL_OP_REGIONS[0].clone()
}

/// The type of a unary function
pub fn unary_ty() -> VarId<Pi> {
    LOGICAL_OP_TYS[0].clone()
}

/// The type of a binary function
pub fn binary_ty() -> VarId<Pi> {
    LOGICAL_OP_TYS[1].clone()
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
    /// Get the arity of this logical value
    #[inline]
    pub fn arity(&self) -> u8 {
        self.arity
    }
    /// Get the data of this logical value
    #[inline]
    pub fn data(&self) -> u128 {
        self.data
    }
    /// Get the constant value of this logical operation, if it is indeed a constant
    #[inline]
    pub fn get_const(&self) -> Option<bool> {
        if self.data == LOGICAL_OP_ARITY_MASKS[self.arity as usize] {
            Some(true)
        } else if self.data == 0 {
            Some(false)
        } else {
            None
        }
    }
    /// Check if this value is a constant
    #[inline]
    pub fn is_const(&self) -> bool {
        self.get_const().is_some()
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
    /// Create a new arity-4 logical operation
    #[inline]
    pub fn arity_4(data: u16) -> Logical {
        Self::try_new(4, data as u128).expect("Arity-4 operations are valid")
    }
    /// Create a new arity-5 logical operation
    #[inline]
    pub fn arity_5(data: u32) -> Logical {
        Self::try_new(5, data as u128).expect("Arity-5 operations are valid")
    }
    /// Create a new arity-6 logical operation
    #[inline]
    pub fn arity_6(data: u64) -> Logical {
        Self::try_new(6, data as u128).expect("Arity-6 operations are valid")
    }
    /// Create a new arity-7 logical operation
    #[inline]
    pub fn arity_7(data: u128) -> Logical {
        Self::try_new(7, data).expect("Arity-7 operations are valid")
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
                arity,
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

impl<T: Into<u8>> Index<T> for Logical {
    type Output = bool;
    #[inline]
    fn index(&self, ix: T) -> &bool {
        if self.get_bit(ix.try_into().unwrap()) {
            &true
        } else {
            &false
        }
    }
}

impl BitAnd for Logical {
    type Output = Result<Logical, ()>;

    fn bitand(self, other: Self) -> Result<Logical, ()> {
        if self.arity != other.arity {
            return Err(());
        }
        Ok(Logical {
            data: self.data & other.data,
            arity: self.arity,
        })
    }
}

impl BitOr for Logical {
    type Output = Result<Logical, ()>;

    fn bitor(self, other: Self) -> Result<Logical, ()> {
        if self.arity != other.arity {
            return Err(());
        }
        Ok(Logical {
            data: self.data | other.data,
            arity: self.arity,
        })
    }
}

impl BitXor for Logical {
    type Output = Result<Logical, ()>;

    fn bitxor(self, other: Self) -> Result<Logical, ()> {
        if self.arity != other.arity {
            return Err(());
        }
        Ok(Logical {
            data: self.data ^ other.data,
            arity: self.arity,
        })
    }
}

impl NotOp for Logical {
    type Output = Logical;

    fn not(self) -> Logical {
        Logical {
            data: !self.data & LOGICAL_OP_ARITY_MASKS[self.arity as usize],
            arity: self.arity,
        }
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
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        _ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        // Null evaluation
        if args.is_empty() {
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
                    Either::Left(b) => return Ok(Application::Success(&[], b.into())),
                    Either::Right(f) => l = f,
                }
            }
        }
        if cut_ix == args.len() {
            Ok(Application::Success(&[], l.into()))
        } else {
            let lifetimes = args[cut_ix..].iter().map(|arg| arg.lifetime());
            let lifetime = Lifetime::default()
                .sep_conjs(lifetimes)
                .map_err(|_| Error::LifetimeError)?;
            Ok(Application::Incomplete(lifetime, Bool.into()))
        }
    }
}

trivial_substitute!(Logical);

impl From<Logical> for NormalValue {
    fn from(l: Logical) -> NormalValue {
        NormalValue(ValueEnum::Logical(l))
    }
}

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
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Logical(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for Logical {}

trivial_lifetime!(Logical);

debug_from_display!(Logical);
display_pretty!(Logical);
enum_convert! {
    impl InjectionRef<ValueEnum> for Logical {}
    impl TryFrom<NormalValue> for Logical { as ValueEnum, }
    impl TryFromRef<NormalValue> for Logical { as ValueEnum, }
}

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
        trivial_lifetime!($t);
        trivial_substitute!($t);
        normal_valid!($t);
        impl Apply for $t {
            fn apply_in<'a>(
                &self,
                args: &'a [ValId],
                ctx: &mut Option<EvalCtx>
            ) -> Result<Application<'a>, Error> {
                Logical::from(*self).apply_in(args, ctx)
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
            #[inline]
            fn into_enum(self) -> ValueEnum {
                self.into()
            }
            #[inline]
            fn into_norm(self) -> NormalValue {
                self.into()
            }
        }
        impl ValueData for $t {}
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
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use crate::tokens::{KEYWORD_FALSE, KEYWORD_TRUE};
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

    #[test]
    fn basic_boolean_properties() {
        assert!(Bool.is_ty());
        assert!(!true.is_ty());
        assert!(!false.is_ty());
        assert_eq!(true.ty(), Bool.into_val());
        assert_eq!(false.ty(), Bool.into_val());
        assert_eq!(Bool.into_val(), Bool.into_val());
        assert_eq!(true.into_val(), true.into_val());
        assert_eq!(false.into_val(), false.into_val());
        assert_eq!(Bool.no_deps(), 0);
        assert_eq!(true.no_deps(), 0);
        assert_eq!(false.no_deps(), 0);
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

    #[test]
    #[allow(clippy::eq_op)]
    fn bitwise_on_binary_operations_work() {
        let binary_ops = (0b0000..=0b1111).map(|b| Logical::try_new(2, b).unwrap());
        for op in binary_ops {
            assert_eq!(op | op, Ok(op));
            assert_eq!(op & op, Ok(op));
            assert_eq!(op ^ op, Ok(Logical::try_const(2, false).unwrap()));
            assert_eq!(!op | op, Ok(Logical::try_const(2, true).unwrap()));
            assert_eq!(!op & op, Ok(Logical::try_const(2, false).unwrap()));
            assert_eq!(!op ^ op, Ok(Logical::try_const(2, true).unwrap()));
            assert_eq!(op | !op, Ok(Logical::try_const(2, true).unwrap()));
            assert_eq!(op & !op, Ok(Logical::try_const(2, false).unwrap()));
            assert_eq!(op ^ !op, Ok(Logical::try_const(2, true).unwrap()));
        }
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn is_const_works() {
        for arity in 1..=7 {
            let tc = Logical::try_const(arity, true).unwrap();
            let fc = Logical::try_const(arity, false).unwrap();
            let nc_data =
                0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA & LOGICAL_OP_ARITY_MASKS[arity as usize];
            let nc = Logical::try_new(arity, nc_data).unwrap();
            assert_eq!(tc.arity(), arity);
            assert_eq!(tc.data(), LOGICAL_OP_ARITY_MASKS[arity as usize]);
            assert_eq!(fc.arity(), arity);
            assert_eq!(fc.data(), 0);
            assert_eq!(nc.arity(), arity);
            assert_eq!(nc.data(), nc_data);
            assert!(tc.is_const());
            assert!(fc.is_const());
            assert!(!nc.is_const());
            assert_eq!(tc.get_const(), Some(true));
            assert_eq!(fc.get_const(), Some(false));
            assert_eq!(nc.get_const(), None);
            assert_eq!(fc | fc, Ok(fc));
            assert_eq!(fc | tc, Ok(tc));
            assert_eq!(tc | fc, Ok(tc));
            assert_eq!(tc | tc, Ok(tc));
            assert_eq!(fc | nc, Ok(nc));
            assert_eq!(tc | nc, Ok(tc));
            assert_eq!(nc | fc, Ok(nc));
            assert_eq!(nc | tc, Ok(tc));
            assert_eq!(nc | nc, Ok(nc));
            assert_eq!(fc & fc, Ok(fc));
            assert_eq!(fc & tc, Ok(fc));
            assert_eq!(tc & fc, Ok(fc));
            assert_eq!(tc & tc, Ok(tc));
            assert_eq!(fc & nc, Ok(fc));
            assert_eq!(tc & nc, Ok(nc));
            assert_eq!(nc & fc, Ok(fc));
            assert_eq!(nc & tc, Ok(nc));
            assert_eq!(nc & nc, Ok(nc));
            assert_eq!(fc ^ fc, Ok(fc));
            assert_eq!(fc ^ tc, Ok(tc));
            assert_eq!(tc ^ fc, Ok(tc));
            assert_eq!(tc ^ tc, Ok(fc));
            assert_eq!(fc ^ nc, Ok(nc));
            assert_eq!(tc ^ nc, Ok(!nc));
            assert_eq!(nc ^ fc, Ok(nc));
            assert_eq!(nc ^ tc, Ok(!nc));
            assert_eq!(nc ^ nc, Ok(fc));
            assert_eq!(!tc, fc);
            assert_eq!(!fc, tc);
        }
    }
}
