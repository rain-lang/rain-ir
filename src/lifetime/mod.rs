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
use ptr_union::{Enum2, Union2};
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
#[repr(transparent)]
pub struct Lifetime(Option<Union2<Arc<RegionData>, Arc<LifetimeData>>>);

/// A borrow of a `rain` lifetime
#[derive(Debug, Clone, Eq, Default)]
#[repr(transparent)]
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
    /// Check if this lifetime is trivial, i.e. is a region only
    #[inline]
    pub fn is_trivial(&self) -> bool {
        if let Some(ptr) = &self.0 {
            ptr.is_a()
        } else {
            true
        }
    }
    /// Check if this lifetime is the static lifetime, i.e. only the null region
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_none()
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
    /// Get the region underlying this lifetime data
    #[inline]
    pub fn get_region(&self) -> RegionBorrow<'a> {
        if let Some(ptr) = self.0.clone() {
            match ptr.unpack() {
                Enum2::A(region) => RegionBorrow::coerce(Some(region)),
                Enum2::B(lifetime) => lifetime.get().region(),
            }
        } else {
            RegionBorrow::NULL
        }
    }
}

impl Regional for LifetimeBorrow<'_> {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.get_region()
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

impl From<LifetimeData> for Lifetime {
    #[inline]
    fn from(lifetime: LifetimeData) -> Lifetime {
        match lifetime.into_nontrivial() {
            Ok(lifetime) => {
                let arc = LIFETIME_CACHE.cache(lifetime);
                Lifetime(Some(UnionAlign::right(arc)))
            }
            Err(region) => Lifetime(region.into_arc().map(UnionAlign::left)),
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
        self.borrow_lifetime().get_region()
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
    use crate::primitive::logical::Bool;
    use crate::typing::Type;

    #[test]
    fn lifetime_layout() {
        use std::mem::size_of;
        assert_eq!(size_of::<Lifetime>(), size_of::<*const u8>());
        assert_eq!(size_of::<LifetimeBorrow>(), size_of::<*const u8>());
        let null_lifetime = Lifetime::from(Region::NULL);
        assert!(null_lifetime.is_static());
        assert!(null_lifetime.is_trivial());
        let null_borrow = null_lifetime.borrow_lifetime();
        assert_eq!(null_lifetime, Lifetime::STATIC);
        assert_eq!(null_lifetime, null_borrow);
        assert_eq!(null_borrow, LifetimeBorrow::STATIC);
        assert_eq!(null_lifetime, LifetimeBorrow::STATIC);
        assert_eq!(null_lifetime.region(), Region::NULL);
        assert_eq!(null_borrow.region(), Region::NULL);
    }

    #[test]
    fn lifetime_construction() {
        let region = Region::binary(Bool.into_ty());
        let region_lt = Lifetime::from(region.clone());
        assert!(region_lt.is_trivial());
        assert!(!region_lt.is_static());
        let direct_region_lt = Lifetime::from(LifetimeData::from(region));
        assert_eq!(direct_region_lt, region_lt);
    }
}
