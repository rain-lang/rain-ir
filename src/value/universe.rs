/*!
Typing universes
*/
use crate::quick_pretty;
use crate::value::{
    lifetime::{LifetimeBorrow, Live},
    typing::{Type, Typed},
    TypeId, TypeRef,
};
use lazy_static::lazy_static;
use lazycell::AtomicLazyCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

lazy_static! {
    /// An instance of the universe of finite types
    pub static ref FINITE_TY: TypeId = TypeId::assert_normal_ty(Universe::finite());
}

/// A universe of types
#[derive(Debug, Clone)]
pub struct Universe {
    /// The level of this type universe
    level: usize,
    /// The kind of this type universe
    kind: usize,
    /// The type of this universe. Lazily computed to avoid infinite regress
    ty: AtomicLazyCell<TypeId>,
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
                ty: AtomicLazyCell::new(),
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
            ty: AtomicLazyCell::new(),
        }
    }
    /// Create a simple type universe
    pub fn simple() -> Universe {
        Universe {
            level: 1,
            kind: 0,
            ty: AtomicLazyCell::new(),
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
            ty: AtomicLazyCell::new(),
        }
    }
    /// Get a type universe containing this universe's types and this universe
    pub fn enclosing(&self) -> Universe {
        Universe {
            level: self.level + 1,
            kind: self.kind,
            ty: AtomicLazyCell::new(),
        }
    }
    /// Get the type of this universe
    pub fn enclosing_ty(&self) -> Universe {
        Universe {
            level: self.level + 1,
            kind: self.kind + 1,
            ty: AtomicLazyCell::new(),
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
                ty: AtomicLazyCell::new(),
            })
        }
    }
    /// Take the union of this universe and another
    pub fn union(&self, other: Universe) -> Universe {
        Universe {
            level: self.level.max(other.level),
            kind: self.kind.min(other.kind),
            ty: AtomicLazyCell::new(),
        }
    }
    /// Take the union of an iterator of universes with the given universe
    pub fn union_all<I>(&self, iter: I) -> Universe
    where
        I: Iterator<Item = Universe>,
    {
        let mut base = self.clone();
        for universe in iter {
            base = base.union(universe)
        }
        base
    }
}

quick_pretty!(Universe, s, fmt => write!(fmt, "#universe({}, {})", s.level, s.kind));

impl Live for Universe {
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

impl Typed for Universe {
    fn ty(&self) -> TypeRef {
        if let Some(ty) = self.ty.borrow() {
            ty.borrow_ty()
        } else {
            let universe = TypeId::from(self.enclosing());
            let _ = self.ty.fill(universe); // Ignore a failed fill
            self.ty.borrow().expect("Impossible").borrow_ty()
        }
    }
}

impl Type for Universe {
    fn universe(&self) -> Universe {
        self.enclosing()
    }
}

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
