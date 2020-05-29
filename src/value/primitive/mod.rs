/*!
Primitive `rain` values and associated value descriptors
*/

use crate::{debug_from_display, quick_pretty};

/// The unit type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Unit;

quick_pretty!(Unit, "#unit");
debug_from_display!(Unit);

/// The empty type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Empty;

quick_pretty!(Empty, "#empty");
debug_from_display!(Empty);