/*!
A builder for `rain` expressions
*/
use std::hash::Hash;
use crate::util::symbol_table::SymbolTable;
use crate::value::ValId;

/// A builder for `rain` expressions
#[derive(Debug)]
pub struct Builder<S: Hash + Eq> {
    symbols: SymbolTable<S, ValId>
}