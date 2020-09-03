/*!
The `rain` lifetime system
*/

use crate::region::{data::RegionData, Region, RegionBorrow, Regional};
use crate::value::{NormalValue, ValId, VALUE_CACHE};
use dashcache::{DashCache, GlobalCache};
use elysees::UnionAlign;
use elysees::{Arc, ArcBorrow};
use erasable::{ErasedPtr, Thin};
use lazy_static::lazy_static;
use ptr_union::Union2;
use slice_dst::SliceWithHeader;
use smallvec::SmallVec;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

mod group;
pub use group::*;
mod data;
pub use data::*;
mod params;
pub use params::*;

/// A `rain` lifetime
#[derive(Debug, Clone, Eq, Default)]
pub struct Lifetime(Option<Union2<Arc<RegionData>, Arc<LifetimeData>>>);

/// A borrow of a `rain` lifetime
#[derive(Debug, Clone, Eq, Default)]
pub struct LifetimeBorrow<'a>(
    Option<Union2<ArcBorrow<'a, RegionData>, ArcBorrow<'a, LifetimeData>>>,
);

impl Lifetime {
    /// The static `rain` lifetime
    pub const STATIC: Lifetime = Lifetime(None);
    /// Get a pointer to the data underlying this lifetime
    #[inline]
    pub fn as_ptr(&self) -> Option<ErasedPtr> {
        self.0.as_ref().map(Union2::as_untagged_ptr)
    }
    /// Borrow this lifetime
    #[inline]
    pub fn borrow_lifetime(&self) -> LifetimeBorrow {
        unsafe { std::mem::transmute_copy(self) }
    }
}

impl<'a> LifetimeBorrow<'a> {
    /// The static `rain` lifetime
    pub const STATIC: LifetimeBorrow<'static> = LifetimeBorrow(None);
    /// Get a pointer to the data underlying this lifetime
    #[inline]
    pub fn as_ptr(&self) -> Option<ErasedPtr> {
        self.0.as_ref().map(Union2::as_untagged_ptr)
    }
}

impl Deref for LifetimeBorrow<'_> {
    type Target = Lifetime;
    fn deref(&self) -> &Lifetime {
        unsafe { &*(self as *const _ as *const Lifetime) }
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

impl<'a> PartialEq<LifetimeBorrow<'a>> for Lifetime {
    #[inline]
    fn eq(&self, other: &LifetimeBorrow<'a>) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl Hash for Lifetime {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_ptr().hash(hasher)
    }
}

impl<'a> PartialEq for LifetimeBorrow<'a> {
    #[inline]
    fn eq(&self, other: &LifetimeBorrow) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl<'a> PartialEq<Lifetime> for LifetimeBorrow<'a> {
    #[inline]
    fn eq(&self, other: &Lifetime) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl<'a> Hash for LifetimeBorrow<'a> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_ptr().hash(hasher)
    }
}

impl Regional for Lifetime {
    #[inline]
    fn region(&self) -> RegionBorrow {
        if let Some(data) = &self.0 {
            if let Some(region) = data.with_a(|ard| {
                // This is evil and bad but I can't think of a better way...
                let ptr = Arc::as_ptr(ard);
                let borrow = unsafe { ArcBorrow::from_raw(ptr) };
                RegionBorrow::coerce(Some(borrow))
            }) {
                region
            } else {
                data.b().expect("Data is either A or B...").region()
            }
        } else {
            RegionBorrow::NULL
        }
    }
}

impl Drop for Lifetime {
    #[inline]
    fn drop(&mut self) {
        if let Some(ptr) = &self.0 {
            ptr.with_b(|ltd| LIFETIME_CACHE.try_gc_global(ltd));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lifetime_layout() {
        use std::mem::size_of;
        assert_eq!(size_of::<Lifetime>(), size_of::<*const u8>());
        assert_eq!(size_of::<LifetimeBorrow>(), size_of::<*const u8>());
    }
}
