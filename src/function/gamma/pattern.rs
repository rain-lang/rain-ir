/*!
Pattern matching branches

TODO: consider caching patterns
*/
use crate::function::pi::Pi;
use crate::region::{Region, RegionData};
use crate::value::{Error, TypeId, VarRef};
use std::ops::Deref;
use triomphe::Arc;

/// A branch of a gamma node
#[derive(Debug, Clone, Hash, Eq)]
pub struct Pattern(pub Arc<PatternData>);

impl Pattern {
    /// Get the region associated with this pattern, given a function type and a parent
    pub fn region(&self, ty: VarRef<Pi>) -> Result<Region, Error> {
        self.deref().region(ty)
    }
    /// Get the argument type vector associated with this pattern, given a function type
    pub fn arg_tys(&self, ty: VarRef<Pi>) -> Result<Vec<TypeId>, Error> {
        self.deref().arg_tys(ty)
    }
}

impl Deref for Pattern {
    type Target = PatternData;
    #[inline]
    fn deref(&self) -> &PatternData {
        &self.0
    }
}

impl PartialEq for Pattern {
    #[inline]
    fn eq(&self, other: &Pattern) -> bool {
        let l = self.deref();
        let r = other.deref();
        std::ptr::eq(l, r) || l == r
    }
}

impl From<PatternData> for Pattern {
    fn from(pattern: PatternData) -> Pattern {
        Pattern(pattern.into())
    }
}

/// The data underlying a branch of a gamma node
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PatternData {
    /// The wildcard pattern
    Any,
}

impl PatternData {
    /// Get the region associated with this pattern, given an input region and a parent
    pub fn region(&self, ty: VarRef<Pi>) -> Result<Region, Error> {
        Ok(Region::new(RegionData::with(
            self.arg_tys(ty)?.into(),
            ty.def_region().parent().cloned().unwrap_or(Region::NULL)
        )))
    }
    /// Get the argument type vector associated with this pattern, given an input region
    pub fn arg_tys(&self, ty: VarRef<Pi>) -> Result<Vec<TypeId>, Error> {
        match self {
            PatternData::Any => Ok(ty.def_region().param_tys().iter().cloned().collect()),
        }
    }
}
