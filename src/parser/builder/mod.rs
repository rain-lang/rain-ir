/*!
A builder for `rain` expressions
*/
use crate::util::symbol_table::SymbolTable;
use crate::value::ValId;
use ahash::RandomState;
use std::fmt::{self, Debug, Formatter};
use std::hash::{BuildHasher, Hash};

/// A builder for `rain` expressions
pub struct Builder<S: Hash + Eq, B: BuildHasher = RandomState> {
    symbols: SymbolTable<S, ValId, B>,
}

impl<S: Hash + Eq + Debug, B: BuildHasher> Debug for Builder<S, B> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.debug_struct("Builder")
            .field("symbols", &self.symbols)
            .finish()
    }
}
