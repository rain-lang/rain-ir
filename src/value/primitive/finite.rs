/*!
Finite-valued types
*/

use crate::prettyprinter::tokens::*;
use crate::value::{
    eval::Apply,
    lifetime::{LifetimeBorrow, Live},
    typing::{Type, Typed},
    universe::FINITE_TY,
    TypeRef, UniverseRef, ValId, Value, VarId, VarRef,
};
use crate::{debug_from_display, quick_pretty, trivial_substitute};
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

impl Finite {
    /// Get an index into this type. Return an error if out of bounds
    pub fn ix<I: ToPrimitive>(self, ix: I) -> Result<Index, ()> {
        let ix = if let Some(ix) = ix.to_u128() {
            ix
        } else {
            return Err(());
        };
        Index::try_new(self, ix)
    }
}

impl Live for Finite {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
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
}

impl Type for Finite {
    #[inline]
    fn universe(&self) -> UniverseRef {
        FINITE_TY.borrow_var()
    }
    #[inline]
    fn is_universe(&self) -> bool {
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
quick_pretty!(Index, s, fmt => write!(fmt, "{}({})[{}]", KEYWORD_IX, s.ty, s.ix));
trivial_substitute!(Index);

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
}

impl Apply for Index {}

impl Live for Index {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::builder::Builder;
    use crate::value::ValId;
    #[test]
    fn basic_indexing_works() {
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
    fn index_making_works() {
        // Index construction
        let ix20 = Finite(2).ix(0).unwrap();
        let ix21 = Finite(2).ix(1).unwrap();
        assert!(Finite(2).ix(2).is_err());
        let ix10 = Finite(1).ix(0).unwrap();
        assert!(Finite(1).ix(1).is_err());

        // Indices into unequal types are unequal
        assert_ne!(ix20, ix10);
        assert_ne!(ix21, ix10);

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
        assert!(!f1.is_universe());
        assert_eq!(f1.universe(), FINITE_TY.borrow_var());

        // Finite types and indices live for the static lifetime
        assert_eq!(f1.lifetime(), LifetimeBorrow::default());
        assert_eq!(f2.lifetime(), LifetimeBorrow::default());
        assert_eq!(Finite(1).lifetime(), LifetimeBorrow::default());
        assert_eq!(Finite(2).lifetime(), LifetimeBorrow::default());
        assert_eq!(ix10.lifetime(), LifetimeBorrow::default());
        assert_eq!(ix20.lifetime(), LifetimeBorrow::default());
        assert_eq!(ix21.lifetime(), LifetimeBorrow::default());
    }
}
