/*!
Boolean types and logical operations
*/

use crate::quick_display;
use crate::prettyprinter::tokens::*;

/// The type of booleans
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Bool;

quick_display!(Bool, "{}", KEYWORD_BOOL);