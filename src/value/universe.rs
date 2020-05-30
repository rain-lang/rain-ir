/*!
Typing universes
*/
use crate::quick_pretty;
use crate::value::{
    lifetime::{LifetimeBorrow, Live},
    TypeId, ValId, ValueEnum,
};
use lazy_static::lazy_static;
use std::cmp::Ordering;

lazy_static! {
    /// An instance of the universe of finite types
    pub static ref FINITE_TY: TypeId = TypeId(ValId::from(ValueEnum::Universe(Universe::finite())));
}

/// A universe of types
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Universe {
    /// The level of this type universe
    level: usize,
    /// The kind of this type universe
    kind: usize,
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
            Some(Universe { level, kind })
        } else {
            None
        }
    }
    /// Create a finite type universe
    pub fn finite() -> Universe {
        Universe { level: 0, kind: 0 }
    }
    /// Create a simple type universe
    pub fn simple() -> Universe {
        Universe { level: 1, kind: 0 }
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
        }
    }
    /// Get a type universe containing this universe's types and this universe
    pub fn enclosing(&self) -> Universe {
        Universe {
            level: self.level + 1,
            kind: self.kind,
        }
    }
    /// Get the type of this universe
    pub fn enclosing_ty(&self) -> Universe {
        Universe {
            level: self.level + 1,
            kind: self.kind + 1,
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
            })
        }
    }
    /// Take the union of this universe and another
    pub fn union(&self, other: Universe) -> Universe {
        Universe {
            level: self.level.max(other.level),
            kind: self.kind.min(other.kind),
        }
    }
    /// Take the union of an iterator of universes
    pub fn union_all<I, T>(mut iter: I) -> Option<Universe>
    where
        I: Iterator<Item = T>,
        T: Into<Option<Universe>>,
    {
        let mut result = iter.next()?.into()?;
        while let Some(universe) = iter.next() {
            result = result.union(universe.into()?)
        }
        Some(result)
    }
}

quick_pretty!(Universe, s, fmt => write!(fmt, "#universe({}, {})", s.level, s.kind));

impl Live for Universe {
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
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
