/*!
Colors, which are the primary component of lifetimes
*/
use crate::region::{Region, RegionBorrow, Regional};
use crate::typing::Type;
use std::sync::atomic::AtomicIsize;

/// A color, the atom for building up lifetimes
///
/// Note: this is not yet implemented: the `usize` is just a dummy member!
///
/// # Overview
/// The term "color" is in reference to graph coloring: while a lifetime is best thought of as a property of
/// a value, indicating what values it borrows from or uses, values with different lifetimes may have colors
/// in comon. Each color, then, can be thought of as a highlighted region of the `rain` graph, with
/// - A tree at the core representing possible "computational paths" from the color's "root node"
/// - A collection of "borrow DAGs" growing from single nodes in the tree, representing borrows of points along the path
///
/// Colors, unlike lifetimes, are atomic: they cannot be split into components (or rather, their components, known as
/// fields, are not colors) nor composed out of multiple colors. A lifetime is implemented as a set of tagged colors.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Color {
    region: Option<Region>,
    ix: isize,
}

impl Regional for Color {
    fn region(&self) -> Option<RegionBorrow> {
        self.region.as_ref().map(Region::borrow_region)
    }
}

/// A color counter, to get unique color numbers
#[derive(Debug)]
pub struct ColorCounter(AtomicIsize);

impl ColorCounter {
    /// Get the next color, incrementing the counter
    #[inline(always)]
    pub fn next(&self) -> isize {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
    }
}

/// The global color counter
pub static COLOR_COUNTER: ColorCounter = ColorCounter(AtomicIsize::new(-1));

impl Color {
    /// Create a unique new color in a given region
    ///
    /// This color is guaranteed not to be equal to *any* other color (in *any* region)
    #[inline]
    pub fn new_in(region: Option<Region>) -> Color {
        let ix = COLOR_COUNTER.next();
        Color { region, ix }
    }
    /// Create a unique new color in the null region
    ///
    /// This color is guaranteed not to be equal to *any* other color (in *any* region).
    /// This method is equivalent to calling `Color::new_in(None)`.
    #[inline]
    pub fn new() -> Color {
        Self::new_in(None)
    }
    /// Create an *unchecked* color for a parameter to a region. It is a logic error if the parameter index is out of bounds.
    pub fn param_unchecked(region: Region, ix: usize) -> Color {
        debug_assert!(
            region.len() > ix,
            "Parameter index {} out of bounds for region {:#?}",
            ix,
            region
        );
        Color {
            region: Some(region),
            ix: ix as isize,
        }
    }
    /// Create the color of a parameter to a region. Return a `None` if the index is out of bounds or to an unrestricted parameter
    ///
    /// Every parameter to a region which is substructural (i.e. restricted) is assigned a unique color. Calling this function
    /// twice for the same [`Region`](Region) and index is guaranteed to yield the same color.
    pub fn param(region: &Region, ix: usize) -> Option<Color> {
        if ix >= region.len() || !region[ix].is_substruct() {
            return None;
        }
        Some(Color::param_unchecked(region.clone(), ix))
    }
    /// Create the color of a parameter to a region. Return a `None` if the index is out of bounds or to an unrestricted parameter
    ///
    /// Every parameter to a region which is substructural (i.e. restricted) is assigned a unique color. Calling this function
    /// twice for the same [`Region`](Region) and index is guaranteed to yield the same color.
    pub fn make_param(region: Region, ix: usize) -> Option<Color> {
        if ix >= region.len() || !region[ix].is_substruct() {
            return None;
        }
        Some(Color::param_unchecked(region, ix))
    }
    /// Get the parameter index of this color, if any
    pub fn param_ix(&self) -> Option<usize> {
        if self.ix >= 0 {
            Some(self.ix as usize)
        } else {
            None
        }
    }
    /// Check whether this color is a parameter
    pub fn is_param(&self) -> bool {
        self.ix >= 0
    }
}

impl Default for Color {
    #[inline]
    fn default() -> Color {
        Color::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::primitive::logical::Bool;

    #[test]
    fn new_colors_are_not_equal() {
        let unary_region = Region::with(vec![Bool.into()].into(), None).unwrap();
        assert_ne!(Color::new(), Color::new());
        assert_ne!(
            Color::new_in(Some(unary_region.clone())),
            Color::new_in(Some(unary_region))
        );
    }
    #[test]
    fn new_region_color_construction() {
        let unary_region = Region::with(vec![Bool.into()].into(), None).unwrap();
        // Bool is substructural so *does not get a color*
        assert_eq!(Color::param(&unary_region, 0), None);
        assert_eq!(Color::param(&unary_region, 1), None);
    }
}
