/*!
The `rain` lifetime system
*/

use crate::region::{data::RegionData, Region};
use crate::value::Error;
use crate::value::{ValId, ValRef};
use elysees::Arc;
use elysees::UnionAlign;
use erasable::ErasedPtr;
use fxhash::FxBuildHasher;
use indexmap::{map::Entry, IndexMap};
use itertools::{EitherOrBoth, Itertools};
use ptr_union::Union2;
use smallvec::SmallVec;
use std::hash::{Hash, Hasher};
use std::iter::Copied;
use std::ops::Deref;

mod group;
pub use group::*;
mod params;
pub use params::*;
mod ctx;
pub use ctx::*;
mod data;
pub use data::*;

/// A `rain` lifetime
#[derive(Debug, Clone, Eq, Default)]
pub struct Lifetime(Option<Union2<Arc<RegionData>, Arc<LifetimeData>>>);

impl Lifetime {
    /// The static `rain` lifetime
    pub const STATIC: Lifetime = Lifetime(None);
    /// Get a pointer to the data underlying this lifetime
    #[inline]
    pub fn as_ptr(&self) -> Option<ErasedPtr> {
        self.0.as_ref().map(|ptr| ptr.as_untagged_ptr())
    }
}

impl From<Region> for Lifetime {
    #[inline]
    fn from(region: Region) -> Lifetime {
        if let Some(arc) = region.into_arc() {
            Lifetime(Some(UnionAlign::left(arc)))
        } else {
            Lifetime::STATIC
        }
    }
}

impl PartialEq for Lifetime {
    #[inline]
    fn eq(&self, other: &Lifetime) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl Hash for Lifetime {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_ptr().hash(hasher)
    }
}
