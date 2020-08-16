/*!
The primitive hierarchy of kinds
*/
use crate::eval::Apply;
use crate::lifetime::{LifetimeBorrow, Live};
use crate::typing::{Kind, Type, Typed};
use crate::value::{KindId, NormalValue, TypeRef, ValId, Value, ValueEnum, VarId};
use crate::{enum_convert, lifetime_region, trivial_substitute};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// The kind of mere propositions
///
/// This kind is the union of all unit and empty types. Note that it is *not* equivalent to a boolean type, since we are working in intuitionistic
/// logic and hence do *not* have access to LEM to prove it so.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Prop;

/// The kind of finite types
///
/// This kind is the union of all types which are non-recursive, and hence in particular can only have finitely many values. This kind is closed
/// under products, sums, and function types, making it the smallest *typing universe*
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Fin;

/// The kind of n-sets
///
/// The kind of 0-sets consists of all standard recursive types which do not reference types. 1-sets are allowed to reference 0-sets, and so on.
/// This kind is called "Set" and not "Kind" since it is subject to an important restriction, namely that it's members are "sets" in terms of
/// HoTT, i.e. have their identity type families be unit types.
///
/// Note `rain`'s standard typing universe does *not* obey univalence, so e.g. in `rain` we have `Id(bool, bool) = ()`, *not* `bool`. This is
/// because we treat types more like `(type, representation, label)` pairs for the purposes of low-level programming. In a sense, then, we can
/// view Set(n) as the product of the 0-truncation of Type(n) from HoTT with `(representation, label)` pairs.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Set {
    n: usize,
    succ: OnceCell<VarId<Set>>,
}

impl Set {
    /// Construct a new representative of the n-sets
    pub fn new(n: usize) -> Set {
        Set {
            n,
            succ: OnceCell::new(),
        }
    }
}
// Constants:

lazy_static! {
    /// The kind of mere propositions
    pub static ref PROP: VarId<Prop> = VarId::direct_new(Prop);
    /// The kind of finite types
    pub static ref FIN: VarId<Fin> = VarId::direct_new(Fin);
    /// The kind of sets, i.e. 0-types
    pub static ref SET: VarId<Set> = VarId::direct_new(Set::new(0));
}

// Value implementations:

enum_convert! {
    impl InjectionRef<ValueEnum> for Prop {}
    impl TryFrom<NormalValue> for Prop { as ValueEnum, }
    impl TryFromRef<NormalValue> for Prop { as ValueEnum, }
    impl InjectionRef<ValueEnum> for Fin {}
    impl TryFrom<NormalValue> for Fin { as ValueEnum, }
    impl TryFromRef<NormalValue> for Fin { as ValueEnum, }
    impl InjectionRef<ValueEnum> for Set {}
    impl TryFrom<NormalValue> for Set { as ValueEnum, }
    impl TryFromRef<NormalValue> for Set { as ValueEnum, }
}

impl From<Prop> for NormalValue {
    fn from(prop: Prop) -> NormalValue {
        NormalValue(ValueEnum::Prop(prop))
    }
}

impl From<Fin> for NormalValue {
    fn from(fin: Fin) -> NormalValue {
        NormalValue(ValueEnum::Fin(fin))
    }
}

impl From<Set> for NormalValue {
    fn from(set: Set) -> NormalValue {
        NormalValue(ValueEnum::Set(set))
    }
}

impl Typed for Prop {
    #[inline]
    fn ty(&self) -> TypeRef {
        unimplemented!("Fin into val")
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
    #[inline]
    fn is_kind(&self) -> bool {
        true
    }
}

impl Live for Prop {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::STATIC
    }
}

lifetime_region!(Prop);
trivial_substitute!(Prop);

impl Apply for Prop {}

impl Value for Prop {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "{:?} has no dependencies, but attempted to get no #{}",
            self, ix
        )
    }
    #[inline]
    fn into_val(self) -> ValId {
        PROP.clone_val()
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl Type for Prop {
    #[inline]
    fn is_affine(&self) -> bool {
        //TODO: think about this...
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        //TODO: think about this
        false
    }
}

impl Kind for Prop {
    #[inline]
    fn id_kind(&self) -> KindId {
        Prop.into_kind()
    }
}

impl Typed for Fin {
    #[inline]
    fn ty(&self) -> TypeRef {
        unimplemented!("Set-0 into val")
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
    #[inline]
    fn is_kind(&self) -> bool {
        true
    }
}

impl Live for Fin {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::STATIC
    }
}

lifetime_region!(Fin);
trivial_substitute!(Fin);

impl Apply for Fin {}

impl Value for Fin {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "{:?} has no dependencies, but attempted to get no #{}",
            self, ix
        )
    }
    #[inline]
    fn into_val(self) -> ValId {
        FIN.clone_val()
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl Type for Fin {
    #[inline]
    fn is_affine(&self) -> bool {
        //TODO: think about this...
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        //TODO: think about this
        false
    }
}

impl Kind for Fin {
    #[inline]
    fn id_kind(&self) -> KindId {
        Prop.into_kind()
    }
}

impl Typed for Set {
    #[inline]
    fn ty(&self) -> TypeRef {
        unimplemented!("Set-n into val")
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
    #[inline]
    fn is_kind(&self) -> bool {
        true
    }
}

impl Live for Set {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::STATIC
    }
}

lifetime_region!(Set);
trivial_substitute!(Set);

impl Apply for Set {}

impl Value for Set {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "{:?} has no dependencies, but attempted to get no #{}",
            self, ix
        )
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl Type for Set {
    #[inline]
    fn is_affine(&self) -> bool {
        //TODO: think about this...
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        //TODO: think about this
        false
    }
}

impl Kind for Set {
    #[inline]
    fn id_kind(&self) -> KindId {
        Prop.into_kind()
    }
}

// General implementations:

impl Hash for Set {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.n.hash(hasher)
    }
}

impl PartialOrd for Set {
    #[inline]
    fn partial_cmp(&self, other: &Set) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Set {
    #[inline]
    fn cmp(&self, other: &Set) -> Ordering {
        self.n.cmp(&other.n)
    }
}

// Prettyprinting:
#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Prop {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(Prop prettyprinting unimplemented)")
        }
    }
    impl PrettyPrint for Fin {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(Fin prettyprinting unimplemented)")
        }
    }
    impl PrettyPrint for Set {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(Set prettyprinting unimplemented)")
        }
    }
}
