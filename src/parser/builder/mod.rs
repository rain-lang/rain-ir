/*!
A builder for `rain` expressions
*/
use crate::util::symbol_table::SymbolTable;
use crate::value::{ValId, tuple::Tuple, expr::Sexpr};
use super::ast::{Ident, Expr, Member, Tuple as TupleExpr, Sexpr as SExpr};
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

/// An error building a `rain` expression
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error<'a> {
    /// An undefined identifier
    UndefinedIdent(Ident<'a>),
    /// An unimplemented `rain` IR build
    NotImplemented
}

impl<'a, S: Hash + Eq + From<&'a str>, B: BuildHasher> Builder<S, B> {
    /// Build a `rain` expression into IR
    pub fn build_expr(&self, expr: &Expr<'a>) -> Result<ValId, Error<'a>> {
        let result_value = match expr {
            Expr::Ident(ident) => self.build_ident(*ident)?,
            Expr::Member(member) => self.build_member(member)?,
            Expr::Sexpr(sexpr) => self.build_sexpr(sexpr)?.into(),
            Expr::Tuple(tuple) => self.build_tuple(tuple)?.into()
        };
        Ok(result_value)
    }
    /// Build a `rain` ident
    pub fn build_ident(&self, _ident: Ident<'a>) -> Result<ValId, Error<'a>> {
        Err(Error::NotImplemented)
    }
    /// Build a member expression
    pub fn build_member(&self, _member: &Member<'a>) -> Result<ValId, Error<'a>> {
        Err(Error::NotImplemented)
    }
    /// Build an S-expression
    pub fn build_sexpr(&self, _sexpr: &SExpr<'a>) -> Result<Sexpr, Error<'a>> {
        Err(Error::NotImplemented)
    }
    /// Build a tuple
    pub fn build_tuple(&self, _tuple: &TupleExpr<'a>) -> Result<Tuple, Error<'a>> {
        Err(Error::NotImplemented)
    } 
}
