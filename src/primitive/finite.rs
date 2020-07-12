/*!
Finite-valued types.
*/
use crate::eval::Apply;
use crate::tokens::*;
use crate::typing::{Type, Typed};
use crate::value::{
    universe::FINITE_TY, NormalValue, TypeRef, UniverseRef, ValId, Value, ValueData, ValueEnum,
    VarId, VarRef,
};
use crate::{debug_from_display, enum_convert, quick_pretty, trivial_lifetime, trivial_substitute};
use num::ToPrimitive;
use ref_cast::RefCast;
use std::cmp::Ordering;
use std::ops::Deref;

/// A type with `n` values.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, RefCast)]
#[repr(transparent)]
pub struct Finite(pub u128);

debug_from_display!(Finite);
quick_pretty!(Finite, s, fmt => write!(fmt, "{}({})", KEYWORD_FINITE, s.0));
trivial_lifetime!(Finite);
trivial_substitute!(Finite);

enum_convert! {
    impl InjectionRef<ValueEnum> for Finite {}
    impl TryFrom<NormalValue> for Finite { as ValueEnum, }
    impl TryFromRef<NormalValue> for Finite { as ValueEnum, }
}

impl Finite {
    /// Get an index into this type. Return an error if out of bounds.
    pub fn ix<I: ToPrimitive>(self, ix: I) -> Result<Index, ()> {
        let ix = if let Some(ix) = ix.to_u128() {
            ix
        } else {
            return Err(());
        };
        Index::try_new(self, ix)
    }
}

impl Typed for Finite {
    #[inline]
    fn ty(&self) -> TypeRef {
        FINITE_TY.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Apply for Finite {}

impl Value for Finite {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Tried to get dependency #{} of finite type {}, which has none",
            ix, self
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Finite(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for Finite {}

impl Type for Finite {
    #[inline]
    fn universe(&self) -> UniverseRef {
        FINITE_TY.borrow_var()
    }
    #[inline]
    fn is_universe(&self) -> bool {
        false
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

/// An index into a finite type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Index {
    /// The type this index is part of
    ty: VarId<Finite>,
    /// This index
    ix: u128,
}

impl PartialOrd for Index {
    /**
    Compare two arbitrary indices.

    Indices in general form a partial order: indices into different types are incomparable, whereas
    the trivial injection into the natural numbers induces a total order on indices into the same
    type.
    */
    fn partial_cmp(&self, other: &Index) -> Option<Ordering> {
        if self.ty != other.ty {
            None
        } else {
            Some(self.ix.cmp(&other.ix))
        }
    }
}

debug_from_display!(Index);
quick_pretty!(Index, s, fmt => write!(fmt, "{}({})[{}]", KEYWORD_IX, s.ty.0, s.ix));
trivial_substitute!(Index);
trivial_lifetime!(Index);
enum_convert! {
    impl InjectionRef<ValueEnum> for Index {}
    impl TryFrom<NormalValue> for Index { as ValueEnum, }
    impl TryFromRef<NormalValue> for Index { as ValueEnum, }
}

impl Index {
    /// Try to make a new index into a finite type. Return an error if out of bounds.
    pub fn try_new<F: Into<VarId<Finite>>>(ty: F, ix: u128) -> Result<Index, ()> {
        let ty = ty.into();
        if ix >= ty.deref().0 {
            Err(())
        } else {
            Ok(Index { ty, ix })
        }
    }
    /// Get this index.
    pub fn ix(&self) -> u128 {
        self.ix
    }
    /// Get the (finite) type of this index.
    pub fn get_ty(&self) -> VarRef<Finite> {
        self.ty.borrow_var()
    }
}

impl Typed for Index {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
}

impl Apply for Index {}

impl From<Finite> for NormalValue {
    fn from(finite: Finite) -> NormalValue {
        NormalValue(ValueEnum::Finite(finite))
    }
}

impl From<Index> for NormalValue {
    fn from(ix: Index) -> NormalValue {
        NormalValue(ValueEnum::Index(ix))
    }
}

impl Value for Index {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Tried to get dependency #{} of finite index {}, which has none",
            ix, self
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Index(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for Index {}

#[cfg(feature = "rand")]
mod rand_impl {
    use super::*;
    use rand::distributions::{Distribution, Standard};
    use rand::Rng;

    impl Distribution<Finite> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Finite {
            Finite(rng.gen())
        }
    }

    impl Distribution<Index> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Index {
            let ty = Finite(rng.gen::<u128>().max(1));
            ty.ix(rng.gen_range(0, ty.0)).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifetime::{LifetimeBorrow, Live};
    #[test]
    #[cfg(feature = "parser")]
    fn basic_indexing_works() {
        use crate::builder::Builder;
        use crate::value::ValId;
        let mut builder = Builder::<&str>::new();
        let exprs: &[(&str, ValId)] = &[
            ("[#true #false ()] #ix(3)[1]", false.into()),
            ("[#false [#true] ()] #ix(3)[1] #ix(1)[0]", true.into()),
            ("[#false #finite(6) #false] #ix(3)[1]", Finite(6).into()),
        ];
        for (expr, value) in exprs {
            let (rest, expr) = builder.parse_expr(expr).expect(expr);
            assert_eq!(&expr, value);
            assert_eq!(rest, "");
        }
    }
    #[test]
    fn indices_work() {
        // Index construction.
        let ix20 = Finite(2).ix(0).unwrap();
        let ix21 = Finite(2).ix(1).unwrap();
        assert!(Finite(2).ix(2).is_err());
        let ix10 = Finite(1).ix(0).unwrap();
        assert!(Finite(1).ix(1).is_err());

        // Index printing.
        assert_eq!(format!("{}", ix20), "#ix(2)[0]");
        assert_eq!(format!("{}", ix21), "#ix(2)[1]");
        assert_eq!(format!("{}", ix10), "#ix(1)[0]");

        // Indices into unequal types are unequal.
        assert_ne!(ix20, ix10);
        assert_ne!(ix21, ix10);

        // Indices into unequal types are incomparable.
        assert_eq!(ix20.partial_cmp(&ix10), None);
        assert_eq!(ix21.partial_cmp(&ix10), None);
        assert_eq!(ix10.partial_cmp(&ix20), None);
        assert_eq!(ix10.partial_cmp(&ix21), None);

        // Indices into the same type compare properly.
        assert_eq!(ix20.partial_cmp(&ix20), Some(Ordering::Equal));
        assert_eq!(ix20.partial_cmp(&ix21), Some(Ordering::Less));
        assert_eq!(ix21.partial_cmp(&ix20), Some(Ordering::Greater));
        assert_eq!(ix21.partial_cmp(&ix21), Some(Ordering::Equal));

        let f2 = VarId::<Finite>::from(Finite(2));
        let f1 = VarId::<Finite>::from(Finite(1));

        // Finite types have the right types.
        assert_eq!(ix20.get_ty(), f2);
        assert_eq!(ix20.ty(), f2);
        assert_eq!(ix21.get_ty(), f2);
        assert_eq!(ix21.ty(), f2);
        assert_eq!(ix10.get_ty(), f1);
        assert_eq!(ix10.ty(), f1);
        assert_ne!(f1, f2);

        // Finite types and indices have no dependences.
        assert_eq!(f1.no_deps(), 0);
        assert_eq!(Finite(1).no_deps(), 0);
        assert_eq!(ix10.no_deps(), 0);
        assert_eq!(f2.no_deps(), 0);
        assert_eq!(Finite(2).no_deps(), 0);
        assert_eq!(ix20.no_deps(), 0);
        assert_eq!(ix21.no_deps(), 0);

        // Finite types are types but not universes, indices are not types.
        assert!(f1.is_ty());
        assert!(!ix10.is_ty());
        assert!(!f1.is_universe());
        assert_eq!(f1.universe(), FINITE_TY.borrow_var());

        // Finite types and indices live for the static lifetime.
        assert_eq!(f1.lifetime(), LifetimeBorrow::default());
        assert_eq!(f2.lifetime(), LifetimeBorrow::default());
        assert_eq!(Finite(1).lifetime(), LifetimeBorrow::default());
        assert_eq!(Finite(2).lifetime(), LifetimeBorrow::default());
        assert_eq!(ix10.lifetime(), LifetimeBorrow::default());
        assert_eq!(ix20.lifetime(), LifetimeBorrow::default());
        assert_eq!(ix21.lifetime(), LifetimeBorrow::default());
    }
}
