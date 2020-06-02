/*!
A builder for `rain` expressions
*/
use super::ast::{
    Detuple, Expr, Ident, Index as IndexExpr, Let, Member, Pattern, Sexpr as SExpr, Simple,
    Tuple as TupleExpr, TypeOf,
};
use crate::util::symbol_table::SymbolTable;
use crate::value::{
    expr::Sexpr,
    primitive::{finite::Index, Unit, UNIT},
    tuple::Tuple,
    typing::Typed,
    TypeId, ValId, ValueEnum,
};
use ahash::RandomState;
use num::ToPrimitive;
use std::borrow::Borrow;
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
            symbols: SymbolTable::new(),
        }
    }
}

impl<'a, S: Hash + Eq + From<&'a str>, B: BuildHasher + Default> Default for Builder<S, B> {
    fn default() -> Builder<S, B> {
        Builder {
            symbols: SymbolTable::default(),
        }
    }
}

/// An error building a `rain` expression
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error<'a> {
    /// An undefined identifier
    UndefinedIdent(Ident<'a>),
    /// An identifier which cannot be assigned to
    CannotAssignIdent(Ident<'a>),
    /// A tuple size mismatch
    TupleSizeMismatch {
        /// The tuple size obtained
        got: usize,
        /// The expected tuple size
        expected: usize,
    },
    /// Index out of bounds
    IndexOutOfBounds {
        /// The index
        ix: u128,
        /// The maximum index
        max: usize,
    },
    /// A type mismatch
    TypeMismatch {
        /// The tuple size obtained
        got: TypeId,
        /// The expected tuple size
        expected: TypeId,
    },
    /// An error message
    Message(&'a str),
    /// An unimplemented `rain` IR build
    NotImplemented(&'a str),
}

impl<'a, S: Hash + Eq + Borrow<str> + From<&'a str>, B: BuildHasher> Builder<S, B> {
    /// Build a `rain` expression into IR
    pub fn build_expr(&mut self, expr: &Expr<'a>) -> Result<ValId, Error<'a>> {
        let result_value = match expr {
            Expr::Ident(ident) => self.build_ident(*ident)?.clone(),
            Expr::Member(member) => self.build_member(member)?,
            Expr::Sexpr(sexpr) => self.build_sexpr(sexpr)?.into(),
            Expr::Tuple(tuple) => self.build_tuple(tuple)?.into(),
            Expr::Bool(b) => (*b).into(),
            Expr::BoolTy(b) => (*b).into(),
            Expr::TypeOf(ty) => self.build_typeof(ty)?.into(),
            Expr::Finite(f) => (*f).into(),
            Expr::Index(i) => self.build_index(*i)?.into(),
        };
        Ok(result_value)
    }

    /// Build a `rain` typeof expression
    pub fn build_typeof(&mut self, ty: &TypeOf<'a>) -> Result<TypeId, Error<'a>> {
        let expr = self.build_expr(&ty.0)?;
        Ok(expr.ty().clone_ty())
    }

    /// Build a `rain` ident
    pub fn build_ident(&mut self, ident: Ident<'a>) -> Result<&ValId, Error<'a>> {
        let sym = ident
            .get_sym()
            .map_err(|_| Error::NotImplemented("Non-symbolic idents not implemented!"))?
            .ok_or(Error::Message("Cannot access the null identifier"))?;
        self.symbols.get(sym).ok_or(Error::UndefinedIdent(ident))
    }

    /// Build an index into a finite type
    pub fn build_index(&self, ix: IndexExpr) -> Result<Index, Error<'a>> {
        if let Some(ty) = ix.ty {
            Index::try_new(ty, ix.ix).map_err(|_| Error::Message("Invalid index!"))
        } else {
            Err(Error::NotImplemented(
                "Index type inference not implemented!",
            ))
        }
    }

    /// Build a member expression
    pub fn build_member(&mut self, member: &Member<'a>) -> Result<ValId, Error<'a>> {
        let mut base = self.build_expr(&member.base)?;
        for ident in member.path.iter() {
            if let Ok(ix) = ident.get_u128() {
                // First try direct tuple indexing
                match base.as_enum() {
                    ValueEnum::Tuple(t) => {
                        let ix = if let Some(ix_u) = ix.to_usize() {
                            if ix_u < t.len() {
                                ix_u
                            } else {
                                return Err(Error::IndexOutOfBounds { ix, max: t.len() });
                            }
                        } else {
                            return Err(Error::IndexOutOfBounds { ix, max: t.len() });
                        };
                        base = t[ix].clone();
                    }
                    _ => match base.ty().as_enum() {
                        // Else try index-expression building
                        ValueEnum::Product(p) => {
                            let _ix = if let Some(ix_u) = ix.to_usize() {
                                if ix_u < p.len() {
                                    ix_u
                                } else {
                                    return Err(Error::IndexOutOfBounds { ix, max: p.len() });
                                }
                            } else {
                                return Err(Error::IndexOutOfBounds { ix, max: p.len() });
                            };
                            //TODO: build appropriate sexpr
                            unimplemented!()
                        }
                        _ => return Err(Error::Message("Non-tuple indexing not yet implemented!")),
                    },
                }
            } else {
                return Err(Error::Message(
                    "Non-numeric member-access not yet implemented!",
                ));
            }
        }
        Ok(base)
    }

    /// Build an S-expression
    pub fn build_sexpr(&mut self, sexpr: &SExpr<'a>) -> Result<Sexpr, Error<'a>> {
        match sexpr.len() {
            0 => Ok(Sexpr::unit()),
            1 => Ok(Sexpr::singleton(self.build_expr(&sexpr.0[0])?)),
            _ => Err(Error::NotImplemented(
                "Non-singleton/unit sexpr building is not implemented",
            )),
        }
    }

    /// Build a tuple
    pub fn build_tuple(&mut self, tuple: &TupleExpr<'a>) -> Result<Tuple, Error<'a>> {
        let elems: Result<_, _> = tuple.0.iter().map(|elem| self.build_expr(elem)).collect();
        Tuple::new(elems?).map_err(|_| Error::Message("Failed to build tuple"))
    }

    /// Build a let-statement
    pub fn build_let(&mut self, l: &Let<'a>) -> Result<(), Error<'a>> {
        let rhs = self.build_expr(&l.rhs)?;
        self.build_assign(&l.lhs, rhs)
    }

    /// Build an assignment
    pub fn build_assign(&mut self, p: &Pattern<'a>, v: ValId) -> Result<(), Error<'a>> {
        match p {
            Pattern::Simple(s) => self.build_simple(s, v),
            Pattern::Detuple(d) => self.build_detuple(d, v),
        }
    }
    /// Build a simple assignment
    pub fn build_simple(&mut self, s: &Simple<'a>, v: ValId) -> Result<(), Error<'a>> {
        if let Some(_ty) = s.ty.as_ref() {
            return Err(Error::NotImplemented(
                "Simple-assignment type checking not yet implemented!",
            ));
        }
        if let Some(var) = s.var.get_sym().map_err(Error::CannotAssignIdent)? {
            //TODO: pattern-assignment
            self.symbols.def(var.into(), v);
        }
        Ok(())
    }
    /// Build a tuple-destructure assignment
    pub fn build_detuple(&mut self, d: &Detuple<'a>, v: ValId) -> Result<(), Error<'a>> {
        match d.0.len() {
            0 => {
                let got = v.ty();
                if got == UNIT.borrow_var() {
                    Ok(())
                } else {
                    Err(Error::TypeMismatch {
                        got: got.clone_ty(),
                        expected: Unit.into(),
                    })
                }
            }
            _ => Err(Error::NotImplemented(
                "Non-unit tuple destructure is not yet implemented!",
            )),
        }
    }
}
