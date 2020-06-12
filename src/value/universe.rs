/*!
Typing universes
*/
use crate::eval::Apply;
use crate::lifetime::{LifetimeBorrow, Live};
use crate::typing::{Type, Typed};
use crate::value::{NormalValue, TypeRef, UniverseId, UniverseRef, ValId, Value, ValueEnum, ValueData};
use crate::{lifetime_region, quick_pretty, trivial_substitute};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

lazy_static! {
    /// An instance of the universe of finite types
    pub static ref FINITE_TY: UniverseId = UniverseId::direct_new(Universe::finite());
}

/// A universe of types
#[derive(Debug, Clone)]
pub struct Universe {
    /// The level of this type universe
    level: usize,
    /// The kind of this type universe
    kind: usize,
    /// The type of this universe. Lazily computed to avoid infinite regress
    ty: OnceCell<UniverseId>,
}

impl PartialEq for Universe {
    #[inline]
    fn eq(&self, other: &Universe) -> bool {
        self.level == other.level && self.kind == other.kind
    }
}

impl Eq for Universe {}

impl Hash for Universe {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_usize(self.level);
        hasher.write_usize(self.kind);
    }
}

impl PartialOrd for Universe {
    #[inline]
    fn partial_cmp(&self, other: &Universe) -> Option<Ordering> {
        use Ordering::*;
        let level_cmp = self.level.cmp(&other.level); // n-universe inclusion
        let kind_cmp = self.kind.cmp(&other.kind).reverse(); // n-kind inclusion
        match (level_cmp, kind_cmp) {
            (Greater, Less) => None,
            (Less, Greater) => None,
            (Equal, ord) => Some(ord),
            (ord, _) => Some(ord),
        }
    }
}

impl Universe {
    /// Try to make a universe from a level and kind
    pub fn try_new(level: usize, kind: usize) -> Option<Universe> {
        if level >= kind {
            Some(Universe {
                level,
                kind,
                ty: OnceCell::new(),
            })
        } else {
            None
        }
    }
    /// Create a finite type universe
    pub fn finite() -> Universe {
        Universe {
            level: 0,
            kind: 0,
            ty: OnceCell::new(),
        }
    }
    /// Create a simple type universe
    pub fn simple() -> Universe {
        Universe {
            level: 1,
            kind: 0,
            ty: OnceCell::new(),
        }
    }
    /// Get the level of this type universe
    pub fn level(&self) -> usize {
        self.level
    }
    /// Get the kind of this type universe
    pub fn kind(&self) -> usize {
        self.kind
    }
    /// Get a type universe at the same level as this one, but which is not a kind
    pub fn base_level(&self) -> Universe {
        Universe {
            level: self.level,
            kind: 0,
            ty: OnceCell::new(),
        }
    }
    /// Get a type universe containing this universe's types and this universe as a `Universe`
    pub fn enclosing(&self) -> Universe {
        Universe {
            level: self.level + 1,
            kind: self.kind,
            ty: OnceCell::new(),
        }
    }
    /// Get the type of this universe as a `Universe`
    pub fn enclosing_ty(&self) -> Universe {
        Universe {
            level: self.level + 1,
            kind: self.kind + 1,
            ty: OnceCell::new(),
        }
    }
    /// Get the universe of elements in this universe, if any
    pub fn enclosed(&self) -> Option<Universe> {
        if self.kind == 0 {
            None
        } else {
            Some(Universe {
                level: self.level,
                kind: self.kind - 1,
                ty: OnceCell::new(),
            })
        }
    }
}

quick_pretty!(Universe, s, fmt => write!(fmt, "#universe({}, {})", s.level, s.kind));
trivial_substitute!(Universe);

impl<'a> UniverseId {
    /// Take the union of this universe and another
    pub fn union(&'a self, other: UniverseRef<'a>) -> UniverseId {
        self.borrow_var().union(other)
    }
    /// Take the union of an iterator of universes with the given universe
    pub fn union_all<I>(&'a self, iter: I) -> UniverseId
    where
        I: Iterator<Item = UniverseRef<'a>>,
    {
        self.borrow_var().union_all(iter)
    }
}

impl<'a> UniverseRef<'a> {
    /// Take the union of this universe and another
    pub fn union(&self, other: UniverseRef<'a>) -> UniverseId {
        //TODO: optimize, `UniverseCow`...
        if self.deref() >= other.deref() {
            self.clone_var()
        } else if other.deref() >= self.deref() {
            other.clone_var()
        } else {
            Universe {
                level: self.level.max(other.level),
                kind: self.kind.min(other.kind),
                ty: OnceCell::new(),
            }
            .into()
        }
    }
    /// Take the union of an iterator of universes with the given universe
    pub fn union_all<I>(&self, iter: I) -> UniverseId
    where
        I: Iterator<Item = UniverseRef<'a>>,
    {
        //TODO: optimize... `UniverseCow`...
        let mut base = self.clone_var();
        for universe in iter {
            base = base.union(universe)
        }
        base
    }
}

impl Live for Universe {
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

lifetime_region!(Universe);

impl Typed for Universe {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.universe().as_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Type for Universe {
    #[inline]
    fn universe(&self) -> UniverseRef {
        if let Some(ty) = self.ty.get() {
            ty.borrow_var()
        } else {
            let universe = UniverseId::from(self.enclosing());
            let _ = self.ty.set(universe); // Ignore a failed fill
            self.ty
                .get()
                .expect("Impossible: this universe's type has just been initialized!")
                .borrow_var()
        }
    }
    #[inline]
    fn is_universe(&self) -> bool {
        true
    }
}

impl Apply for Universe {}

impl Value for Universe {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Attempted to get dependency {} of typing universe {}, but `Universe` has no dependencies!",
            ix, self
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Universe(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for Universe {}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};
    #[test]
    fn primitive_universes_are_ordered_correctly() {
        use Ordering::*;
        assert_eq!(
            Universe::finite().partial_cmp(&Universe::simple()),
            Some(Less)
        );
        assert_eq!(
            Universe::simple().partial_cmp(&Universe::finite()),
            Some(Greater)
        );
        assert_eq!(
            Universe::finite()
                .enclosing_ty()
                .partial_cmp(&Universe::simple().enclosing_ty()),
            Some(Less)
        );
        assert_eq!(
            Universe::simple()
                .enclosing_ty()
                .partial_cmp(&Universe::finite().enclosing_ty()),
            Some(Greater)
        );
        assert_eq!(
            Universe::finite()
                .enclosing_ty()
                .partial_cmp(&Universe::simple()),
            Some(Less)
        );
        assert_eq!(
            Universe::simple().partial_cmp(&Universe::finite().enclosing_ty()),
            Some(Greater)
        );
    }
}
