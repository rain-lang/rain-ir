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
    region: Region,
    ix: isize,
}

impl Regional for Color {
    fn region(&self) -> RegionBorrow {
        self.region.borrow_region()
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
    pub fn new_in(region: Region) -> Color {
        let ix = COLOR_COUNTER.next();
        Color { region, ix }
    }
    /// Create a unique new color in the null region
    pub fn new() -> Color {
        Self::new_in(Region::NULL)
    }
    /// Create the color of a parameter to a region. Return an `None` if the index is out of bounds or to an unrestricted parameter
    pub fn param(region: Region, ix: usize) -> Option<Color> {
        if ix >= region.len() || !region[ix].is_substruct()  {
            return None;
        }
        let ix = ix as isize; // Might get bugs around 2 billion parameters on 32-bit systems, but... lazy...
        Some(Color { region, ix })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new_colors_are_not_equal() {
        assert_ne!(Color::new(), Color::new());
    }
}
