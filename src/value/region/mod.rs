/*!
`rain` node regions
*/

use smallvec::SmallVec;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use triomphe::Arc;

/// The size of a small set of parameters to a `rain` region
pub const SMALL_PARAMS: usize = 2;

/// A `rain` region
#[derive(Debug, Clone, Eq)]
pub struct Region(Arc<RegionData>);

impl PartialEq for Region {
    fn eq(&self, other: &Region) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Region {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

/// A vector of parameter types
pub type ParamTyVec = SmallVec<[(); SMALL_PARAMS]>;

/// The data composing a `rain` region
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RegionData {
    /// The parameter types of this region
    pub param_tys: ParamTyVec,
    /// The parent of this region
    pub parent: Region,
    /// The depth of this region above the null region
    pub depth: usize
}

/**
A parameter to a `rain` region.Hasher

Note that the uniqueness of `ValId`s corresponding to a given parameter is enforced by the hash-consing algorithm.
*/
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parameter {
    /// The region this is a parameter for
    region: Region,
    /// The index of this parameter in the region's type vector
    ix: usize,
}
