/*!
A builder for `rain` expressions
*/
use crate::util::symbol_table::SymbolTable;
use crate::value::ValId;
use ahash::RandomState;
use std::fmt::{self, Debug, Formatter};
use std::hash::{BuildHasher, Hash};

/// A rain IR builder
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


impl<'a, S: Hash + Eq + From<&'a str>> Builder<S> {
    /// Create a new builder
    pub fn new() -> Builder<S> {
        Builder {
            symbols: SymbolTable::new()
        }
    }
}

impl<'a, S: Hash + Eq + From<&'a str>, B: BuildHasher + Default> Default for Builder<S, B> {
    fn default() -> Builder<S, B> {
        Builder {
            symbols: SymbolTable::default()
        }
    }
}

impl<'a, S: Hash + Eq + From<&'a str>, B: BuildHasher> Builder<S, B> {
    //TODO: build it!
}
