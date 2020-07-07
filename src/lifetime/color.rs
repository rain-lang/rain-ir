/*!
Colors, which are the primary component of lifetimes
*/

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
pub struct Color(pub usize);