/*!
The data composing a non-null `rain` region
*/
use super::*;
use crate::typing::{primitive::Prop, Universe};
use crate::value::Error;
use im::Vector;
use std::hash::{Hash, Hasher};

/// The data composing a `rain` region
#[derive(Debug, Clone, Eq)]
pub struct RegionData {
    /// The parents of this region
    pub(super) parents: Vector<Region>,
    /// The parameter types of this region
    pub(super) param_tys: TyArr,
    /// The universe of the parameter types of this region
    pub(super) universe: UniverseId,
}

impl RegionData {
    /// Create data for a new region with a given parameter type vector and a parent region
    ///
    /// This constructor does not check whether all parameter types lie within the given parent region, but it is a *logic error* if they do not!
    /// Similarly, it does not check whether all parameter types lie within the given parent universe, but it is a *logic error* if they do not!
    #[inline]
    pub fn with_unchecked(
        param_tys: TyArr,
        parent: Option<Region>,
        universe: UniverseId,
    ) -> RegionData {
        let parents = if let Some(parent) = parent {
            let mut result = parent.data().parents.clone();
            result.push_back(parent);
            result
        } else {
            Vector::new()
        };
        RegionData {
            param_tys,
            parents,
            universe,
        }
    }
    /// Create data for a new region with a given parameter type vector and a parent region
    #[inline]
    pub fn with(param_tys: TyArr, parent: Option<Region>) -> Result<RegionData, Error> {
        use Ordering::*;
        let mut universe = None;
        for param_ty in param_tys.iter() {
            match param_ty.region().partial_cmp(&parent.region()) {
                None => return Err(Error::IncomparableRegions),
                Some(Greater) => return Err(Error::InvalidCastIntoLifetime),
                _ => {}
            }
        }
        Ok(Self::with_unchecked(
            param_tys,
            parent,
            universe.unwrap_or_else(|| Prop.into_universe()),
        ))
    }
    /// Create data for a new, empty region with an optional parent region
    #[inline]
    pub fn with_parent(parent: Option<Region>) -> RegionData {
        Self::with_unchecked(TyArr::default(), parent, Prop.into_universe())
    }
    /// Get the depth of this region
    #[inline]
    pub fn depth(&self) -> usize {
        self.parents.len() + 1
    }
    /// Get the parent of this region
    #[inline]
    pub fn parent(&self) -> Option<&Region> {
        self.parents.last()
    }
    /// Get the parameter types of this region
    #[inline]
    pub fn param_tys(&self) -> &TyArr {
        &self.param_tys
    }
    /// Get the universe of this region data
    #[inline]
    pub fn universe(&self) -> &UniverseId {
        &self.universe
    }
}

impl Deref for RegionData {
    type Target = [TypeId];
    #[inline]
    fn deref(&self) -> &[TypeId] {
        self.param_tys.deref()
    }
}

impl PartialEq for RegionData {
    fn eq(&self, other: &RegionData) -> bool {
        self.parents.last() == other.parents.last() && self.param_tys == other.param_tys
    }
}

impl Hash for RegionData {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.parents.last().hash(hasher);
        self.param_tys.hash(hasher);
    }
}

impl PartialOrd for RegionData {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionData) -> Option<Ordering> {
        if self.parents.len() == other.parents.len() {
            if self == other {
                Some(Ordering::Equal)
            } else {
                None
            }
        } else {
            let min_ix = self.parents.len().min(other.parents.len());
            if min_ix == self.parents.len() {
                if other.parents[min_ix] == *self {
                    Some(Ordering::Less)
                } else {
                    None
                }
            } else if self.parents[min_ix] == *other {
                Some(Ordering::Greater)
            } else {
                None
            }
        }
    }
}
