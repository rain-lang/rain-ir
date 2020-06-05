/*!
A `rain` evaluation context
*/

use crate::util::symbol_table::SymbolTable;
use crate::value::{NormalValue, ValId};
use std::hash::BuildHasher;
use fxhash::FxBuildHasher;

/// A `rain` evaluation context
#[derive(Debug, Clone, PartialEq)]
pub struct EvalCtx<S: BuildHasher = FxBuildHasher> {
    /// The cache for evaluated values
    cache: SymbolTable<*const NormalValue, ValId, S>,
}

impl<S: BuildHasher + Default> EvalCtx<S> {
    /// Create a new, empty evaluation context with a given capacity
    pub fn with_capacity(n: usize) -> EvalCtx<S> {
        EvalCtx {
            cache: SymbolTable::with_capacity(n)
        }
    }
}
