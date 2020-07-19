/*!
Repetitive impls for `Region`, `Option<Region>`, etc.
*/
use super::*;

impl Regional for Option<Region> {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.as_ref().map(Region::borrow_region)
    }
    /// Get the depth of this object's region
    #[inline]
    fn depth(&self) -> usize {
        self.as_ref().map(Regional::depth).unwrap_or(0)
    }
}

impl Regional for Region {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        Some(self.borrow_region())
    }
    #[inline]
    fn depth(&self) -> usize {
        self.data().depth()
    }
}

impl Regional for Option<RegionBorrow<'_>> {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        *self
    }
    /// Get the depth of this object's region
    #[inline]
    fn depth(&self) -> usize {
        self.as_ref().map(Regional::depth).unwrap_or(0)
    }
}

impl Regional for RegionBorrow<'_> {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        None
    }
    #[inline]
    fn depth(&self) -> usize {
        self.data().depth()
    }
}

// == Comparison ==

impl PartialEq for Region {
    fn eq(&self, other: &Region) -> bool {
        let self_ptr = self.data_ptr();
        let other_ptr = other.data_ptr();
        self_ptr == other_ptr
    }
}

impl PartialEq<RegionBorrow<'_>> for Region {
    fn eq(&self, other: &RegionBorrow) -> bool {
        let self_ptr = self.data_ptr();
        let other_ptr = other.data_ptr();
        self_ptr == other_ptr
    }
}

impl PartialEq<RegionData> for Region {
    fn eq(&self, other: &RegionData) -> bool {
        self.data() == other
    }
}

impl PartialEq for RegionBorrow<'_> {
    fn eq(&self, other: &RegionBorrow) -> bool {
        let self_ptr = self.data_ptr();
        let other_ptr = other.data_ptr();
        self_ptr == other_ptr
    }
}

impl PartialEq<Region> for RegionBorrow<'_> {
    fn eq(&self, other: &Region) -> bool {
        let self_ptr = self.data_ptr();
        let other_ptr = other.data_ptr();
        self_ptr == other_ptr
    }
}

impl PartialEq<RegionData> for RegionBorrow<'_> {
    fn eq(&self, other: &RegionData) -> bool {
        //TODO: pointer check?
        self.data() == other
    }
}

impl PartialOrd for Region {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &Region) -> Option<Ordering> {
        self.data().partial_cmp(&other.data())
    }
}

impl PartialOrd<RegionData> for Region {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionData) -> Option<Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl PartialOrd<RegionBorrow<'_>> for Region {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionBorrow) -> Option<Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl PartialOrd for RegionBorrow<'_> {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionBorrow<'_>) -> Option<Ordering> {
        self.data().partial_cmp(&other.data())
    }
}

impl PartialOrd<RegionData> for RegionBorrow<'_> {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionData) -> Option<Ordering> {
        self.data().partial_cmp(other)
    }
}

impl PartialOrd<Region> for RegionBorrow<'_> {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &Region) -> Option<Ordering> {
        self.data().partial_cmp(&other.data())
    }
}

// == Hashing ==
