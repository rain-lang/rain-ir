/*!
Finite-valued types
*/
use crate::eval::Apply;
use crate::lifetime::Live;
use crate::tokens::*;
use crate::typing::{primitive::FIN, Type, Typed};
use crate::value::{NormalValue, TypeRef, ValId, Value, ValueData, ValueEnum, VarId, VarRef};
use crate::{debug_from_display, enum_convert, quick_pretty, trivial_substitute};
use num::ToPrimitive;
use ref_cast::RefCast;
use std::cmp::Ordering;
use std::ops::Deref;

/// A type with `n` values
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, RefCast)]
#[repr(transparent)]
pub struct Finite(pub u128);

debug_from_display!(Finite);
quick_pretty!(Finite, s, fmt => write!(fmt, "{}({})", KEYWORD_FINITE, s.0));
trivial_substitute!(Finite);
enum_convert! {
    impl InjectionRef<ValueEnum> for Finite {}
    impl TryFrom<NormalValue> for Finite { as ValueEnum, }
    impl TryFromRef<NormalValue> for Finite { as ValueEnum, }
}

impl Finite {
    /// Get an index into this type. Return an error if out of bounds
    pub fn ix<I: ToPrimitive>(self, ix: I) -> Result<Index, ()> {
        match ix.to_u128() {
            Some(ix) if ix < self.0 => Ok(Index {
                ty: self.into_var(),
                ix,
            }),
            _ => Err(()),
        }
    }
    /// Iterate over the members of this finite type
    pub fn iter(self) -> impl Iterator<Item = Index> + DoubleEndedIterator {
        VarId::<Finite>::from(self).owned_iter()
    }
}

impl VarId<Finite> {
    /// Get an index into this type. Return an error if out of bounds
    pub fn into_ix<I: ToPrimitive>(self, ix: I) -> Result<Index, ()> {
        match ix.to_u128() {
            Some(ix) if ix < self.0 => Ok(Index { ty: self, ix }),
            _ => Err(()),
        }
    }
    /// Get an index into this type. Return an error if out of bounds
    pub fn ix<I: ToPrimitive>(&self, ix: I) -> Result<Index, ()> {
        match ix.to_u128() {
            Some(ix) if ix < self.0 => Ok(Index {
                ty: self.clone(),
                ix,
            }),
            _ => Err(()),
        }
    }
    /// Iterate over the members of this finite type
    pub fn iter(&self) -> impl Iterator<Item = Index> + '_ + DoubleEndedIterator {
        (0..(self.0)).map(move |ix| self.clone().ix(ix).expect("Index must be in bounds!"))
    }
    /// Iterate over the members of this finite type
    pub fn owned_iter(self) -> impl Iterator<Item = Index> + DoubleEndedIterator {
        (0..(self.0)).map(move |ix| self.clone().ix(ix).expect("Index must be in bounds!"))
    }
}

impl Live for Finite {}

impl Typed for Finite {
    #[inline]
    fn ty(&self) -> TypeRef {
        FIN.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
    #[inline]
    fn is_kind(&self) -> bool {
        false
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
    fn dep_owned(&self, ix: usize) -> bool {
        panic!(
            "{:?} has no dependencies, but attempted to get no #{}",
            self, ix
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
enum_convert! {
    impl InjectionRef<ValueEnum> for Index {}
    impl TryFrom<NormalValue> for Index { as ValueEnum, }
    impl TryFromRef<NormalValue> for Index { as ValueEnum, }
}

impl Live for Index {}

impl Index {
    /// Try to make a new index into a finite type. Return an error if out of bounds
    pub fn try_new<F: Into<VarId<Finite>>>(ty: F, ix: u128) -> Result<Index, ()> {
        let ty = ty.into();
        if ix >= ty.deref().0 {
            Err(())
        } else {
            Ok(Index { ty, ix })
        }
    }
    /// Get this index
    pub fn ix(&self) -> u128 {
        self.ix
    }
    /// Get the (finite) type of this index
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
    #[inline]
    fn is_kind(&self) -> bool {
        false
    }
}

impl Apply for Index {}

impl From<Finite> for NormalValue {
    fn from(finite: Finite) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Finite(finite))
    }
}

impl From<Index> for NormalValue {
    fn from(ix: Index) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Index(ix))
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
    fn dep_owned(&self, ix: usize) -> bool {
        panic!(
            "{:?} has no dependencies, but attempted to get no #{}",
            self, ix
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
    #[test]
    fn indices_work() {
        // Index construction
        let ix20 = Finite(2).ix(0).unwrap();
        let ix21 = Finite(2).ix(1).unwrap();
        assert!(Finite(2).ix(2).is_err());
        let ix10 = Finite(1).ix(0).unwrap();
        assert!(Finite(1).ix(1).is_err());

        // Index printing
        assert_eq!(format!("{}", ix20), "#ix(2)[0]");
        assert_eq!(format!("{}", ix21), "#ix(2)[1]");
        assert_eq!(format!("{}", ix10), "#ix(1)[0]");

        // Indices into unequal types are unequal
        assert_ne!(ix20, ix10);
        assert_ne!(ix21, ix10);

        // Indices into unequal types are incomparable
        assert_eq!(ix20.partial_cmp(&ix10), None);
        assert_eq!(ix21.partial_cmp(&ix10), None);
        assert_eq!(ix10.partial_cmp(&ix20), None);
        assert_eq!(ix10.partial_cmp(&ix21), None);

        // Indices into the same type compare properly
        assert_eq!(ix20.partial_cmp(&ix20), Some(Ordering::Equal));
        assert_eq!(ix20.partial_cmp(&ix21), Some(Ordering::Less));
        assert_eq!(ix21.partial_cmp(&ix20), Some(Ordering::Greater));
        assert_eq!(ix21.partial_cmp(&ix21), Some(Ordering::Equal));

        let f2 = VarId::<Finite>::from(Finite(2));
        let f1 = VarId::<Finite>::from(Finite(1));

        // Finite types have the right types
        assert_eq!(ix20.get_ty(), f2);
        assert_eq!(ix20.ty(), f2);
        assert_eq!(ix21.get_ty(), f2);
        assert_eq!(ix21.ty(), f2);
        assert_eq!(ix10.get_ty(), f1);
        assert_eq!(ix10.ty(), f1);
        assert_ne!(f1, f2);

        // Finite types and indices have no dependences
        assert_eq!(f1.no_deps(), 0);
        assert_eq!(Finite(1).no_deps(), 0);
        assert_eq!(ix10.no_deps(), 0);
        assert_eq!(f2.no_deps(), 0);
        assert_eq!(Finite(2).no_deps(), 0);
        assert_eq!(ix20.no_deps(), 0);
        assert_eq!(ix21.no_deps(), 0);

        // Finite types are types but not universes, indices are not types
        assert!(f1.is_ty());
        assert!(!ix10.is_ty());
    }
}
