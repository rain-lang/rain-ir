/*!
The `rain` lifetime system
*/

use crate::region::{data::RegionData, Region, RegionBorrow, Regional};
use crate::util::{AddrLookupMut, HasAddr};
use crate::value::{NormalValue, ValAddr, ValId, ValRef, VALUE_CACHE};
use dashcache::{DashCache, GlobalCache};
use elysees::UnionAlign;
use elysees::{Arc, ArcBorrow};
use erasable::{Erasable, ErasedPtr, Thin};
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use lazy_static::lazy_static;
use ptr_union::{Enum2, Union2};
use slice_dst::SliceWithHeader;
use smallvec::SmallVec;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

mod group;
pub use group::*;
mod data;
pub use data::*;
mod params;
pub use params::*;
mod ctx;
pub use ctx::*;

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

/// An object with a lifetime
pub trait Live {
    /// Get the lifetime of this object
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::STATIC
    }
    /// Clone the lifetime of this object
    #[inline]
    fn clone_lifetime(&self) -> Lifetime {
        self.lifetime().clone_lifetime()
    }
}

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
    /// Get the lifetime data behind this lifetime, if any
    #[inline]
    pub fn lt_data(&self) -> Option<&LifetimeData> {
        self.0.as_ref().map(Union2::b).flatten()
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
    /// Get the transient component of this lifetime, if any
    #[inline]
    pub fn is_transient(&self) -> bool {
        self.lt_data()
            .map(LifetimeData::is_transient)
            .unwrap_or(true)
    }
    /// Get the transient component of this lifetime, if any
    #[inline]
    pub fn is_concrete(&self) -> bool {
        self.lt_data()
            .map(LifetimeData::is_concrete)
            .unwrap_or(true)
    }
    /// Get the lender of this lifetime, if any
    #[inline]
    pub fn lender(&self) -> Option<&Group> {
        self.lt_data().map(LifetimeData::lender).flatten()
    }
    /// Get the transient component of this lifetime, if any
    #[inline]
    pub fn transient(&self) -> Option<&Group> {
        self.lt_data().map(LifetimeData::transient).flatten()
    }
    /// Get the concrete component of this lifetime, if any
    #[inline]
    pub fn concrete(&self) -> Lifetime {
        match self.lt_data() {
            Some(data) if !data.is_concrete() => data.concrete().into(),
            _ => self.clone(),
        }
    }
    /// Get the lifetime parameters of this lifetime
    #[inline]
    pub fn params(&self) -> Option<&LifetimeParams> {
        self.lt_data().map(LifetimeData::params)
    }
    /// Check if this lifetime is the static lifetime, i.e. only the null region
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_none()
    }
}

impl Live for Lifetime {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.borrow_lifetime()
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

impl<'a> Live for LifetimeBorrow<'a> {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.clone()
    }
    #[inline]
    fn clone_lifetime(&self) -> Lifetime {
        self.deref().clone()
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

impl<'a> From<RegionBorrow<'a>> for LifetimeBorrow<'a> {
    #[inline]
    fn from(region: RegionBorrow<'a>) -> LifetimeBorrow<'a> {
        LifetimeBorrow(region.get_borrow().map(UnionAlign::left))
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

impl Drop for Lifetime {
    #[inline]
    fn drop(&mut self) {
        if let Some(ptr) = &self.0 {
            ptr.with_b(|ltd| LIFETIME_CACHE.try_gc_global(ltd));
        }
    }
}

impl<T: Live> Regional for T {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.lifetime().get_region()
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
        assert!(null_lifetime.is_transient());
        assert!(null_lifetime.is_concrete());
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
        assert!(region_lt.is_transient());
        assert!(region_lt.is_concrete());
        let direct_region_lt = Lifetime::from(LifetimeData::from(region));
        assert_eq!(direct_region_lt, region_lt);
    }
}
