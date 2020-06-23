/*!
A builder for `rain` expressions
*/
use super::ast::{
    Detuple, Expr, Ident, Index as IndexExpr, Jeq, Lambda as LambdaExpr, Let, Member, ParamArgs,
    Parametrized as ParametrizedExpr, Pattern, Pi as PiExpr, Product as ProductExpr, Scope,
    Sexpr as SExpr, Simple, Statement, Tuple as TupleExpr, TypeOf,
};
use super::{parse_expr, parse_statement};
use crate::function::{lambda::Lambda, pi::Pi};
use crate::primitive::{
    finite::{Finite, Index},
    Unit, UNIT,
};
use crate::region::{Parametrized, Region, RegionData};
use crate::typing::Typed;
use crate::value::{
    self,
    expr::Sexpr,
    tuple::{Product, Tuple},
    TypeId, ValId, ValueEnum,
};
use ahash::RandomState;
use hayami::SymbolTable;
use num::ToPrimitive;
use std::borrow::Borrow;
use std::convert::TryInto;
use std::fmt::{self, Debug, Formatter};
use std::hash::{BuildHasher, Hash};

/// A rain IR builder
pub struct Builder<S: Hash + Eq, B: BuildHasher = RandomState> {
    /// The symbol table
    symbols: SymbolTable<S, ValId, B>,
    /// The stack of regions being defined, along with associated scope stack depth
    stack: Vec<(Region, usize)>,
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
            stack: Vec::new(),
        }
    }
}

impl<'a, S: Hash + Eq + From<&'a str>, B: BuildHasher + Default> Default for Builder<S, B> {
    fn default() -> Builder<S, B> {
        Builder {
            symbols: SymbolTable::default(),
            stack: Vec::new(),
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
    /// A parse error
    ParseError(&'a str),
    /// An error message
    Message(&'a str),
    /// An unimplemented `rain` IR build
    NotImplemented(&'a str),
    /// A value error
    ValueError(&'a str, value::Error),
}

impl<'a> From<value::Error> for Error<'a> {
    fn from(error: value::Error) -> Error<'a> {
        Error::ValueError("Cast:", error)
    }
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
            Expr::Lambda(l) => self.build_lambda(l)?.into(),
            Expr::Pi(p) => self.build_pi(p)?.into(),
            Expr::Product(p) => self.build_product(p)?.into(),
            Expr::Jeq(j) => self.build_jeq(j)?.into(),
            Expr::Scope(s) => self.build_scope(s)?,
            Expr::Logical(l) => (*l).into(),
            Expr::Gamma(_) => unimplemented!("Gamma node construction"),
            Expr::Unit => Unit.into(),
        };
        Ok(result_value)
    }

    /// Build a scope
    pub fn build_scope(&mut self, scope: &Scope<'a>) -> Result<ValId, Error<'a>> {
        self.push_scope();
        for statement in scope.statements.iter() {
            self.build_statement(statement)?
        }
        let result = scope.retv.as_ref().map(|expr| self.build_expr(expr));
        self.pop_scope();
        if let Some(result) = result {
            result
        } else {
            Err(Error::NotImplemented("Non-value scopes"))
        }
    }

    /// Build a `rain` expression into a type. Return an error if it is not
    pub fn build_ty(&mut self, expr: &Expr<'a>) -> Result<TypeId, Error<'a>> {
        self.build_expr(expr)?
            .try_into()
            .map_err(|_| Error::Message("Not a type!"))
    }

    /// Build a `rain` typeof expression
    pub fn build_typeof(&mut self, ty: &TypeOf<'a>) -> Result<TypeId, Error<'a>> {
        let expr = self.build_expr(&ty.0)?;
        Ok(expr.ty().clone_ty())
    }

    /// Build a judgemental equality comparison between a set of `rain` types, returning whether they are all equal
    pub fn build_jeq(&mut self, j: &Jeq<'a>) -> Result<bool, Error<'a>> {
        let mut iter = j.iter().map(|e| self.build_expr(e));
        let mut result = true;
        if let Some(first) = iter.next() {
            let first = first?;
            while let Some(next) = iter.next() {
                if first != next? {
                    result = false
                }
            }
        }
        Ok(result)
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
                            base = Sexpr::try_new(vec![
                                base.clone(),
                                Index::try_new(Finite(p.len() as u128), ix)
                                    .map_err(|_| Error::IndexOutOfBounds {
                                        ix: ix,
                                        max: p.len(),
                                    })?
                                    .into(),
                            ])?
                            .into();
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
        let args: Result<_, _> = sexpr.iter().map(|arg| self.build_expr(arg)).collect();
        Ok(Sexpr::try_new(args?)?)
    }

    /// Build a tuple
    pub fn build_tuple(&mut self, tuple: &TupleExpr<'a>) -> Result<Tuple, Error<'a>> {
        let elems: Result<_, _> = tuple.0.iter().map(|elem| self.build_expr(elem)).collect();
        Tuple::try_new(elems?).map_err(|_| Error::Message("Failed to build tuple"))
    }

    /// Build a product type
    pub fn build_product(&mut self, product: &ProductExpr<'a>) -> Result<Product, Error<'a>> {
        let elems: Result<_, _> = product.0.iter().map(|elem| self.build_ty(elem)).collect();
        Product::try_new(elems?).map_err(|_| Error::Message("Failed to build product"))
    }

    /// Build a let-statement
    pub fn build_let(&mut self, l: &Let<'a>) -> Result<(), Error<'a>> {
        let rhs = self.build_expr(&l.rhs)?;
        self.build_assign(&l.lhs, rhs)
    }

    /// Build a statement
    pub fn build_statement(&mut self, s: &Statement<'a>) -> Result<(), Error<'a>> {
        match s {
            Statement::Let(l) => self.build_let(l),
        }
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
            self.symbols.insert(var.into(), v);
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

    /// Build a set of parameter arguments into a region, registering a new scope for them
    /// Push this region onto the region stack
    pub fn push_args(&mut self, args: &ParamArgs<'a>) -> Result<(), Error<'a>> {
        //TODO: more efficiency for type vector construction...
        let tys: Result<Vec<_>, _> = args.iter().map(|(_, ty)| self.build_ty(ty)).collect();
        let region = Region::new(RegionData::with(
            tys?.into_iter().collect(),
            self.stack
                .last()
                .map(|(r, _)| r.clone())
                .unwrap_or(Region::default()),
        ));
        self.push_scope();
        for (i, (id, _)) in args.iter().enumerate() {
            match id.get_sym() {
                Ok(Some(sym)) => self.symbols.insert(
                    sym.into(),
                    region
                        .clone()
                        .param(i)
                        .expect("Index must be in bounds")
                        .into(),
                ),
                Ok(None) => None,
                Err(_) => {
                    self.pop_scope();
                    return Err(Error::Message("Cannot assign to this symbol!"));
                }
            };
        }
        self.push_region(region);
        Ok(())
    }

    /// Push a region onto the region stack *without affecting the symbol table*
    pub fn push_region(&mut self, region: Region) {
        self.stack.push((region, self.symbols.depth()))
    }

    /// Pop the top region from the region stack, along with any scopes in the region. Return it, if any
    pub fn pop_region(&mut self) -> Option<Region> {
        if let Some((region, depth)) = self.stack.pop() {
            self.symbols.jump(depth);
            Some(region)
        } else {
            None
        }
    }

    /// Push a scope onto the symbol table
    pub fn push_scope(&mut self) {
        self.symbols.push()
    }

    /// Pop a scope from the symbol table
    pub fn pop_scope(&mut self) {
        self.symbols.pop()
    }

    /// Build a lambda function
    pub fn build_lambda(&mut self, lambda: &LambdaExpr<'a>) -> Result<Lambda, Error<'a>> {
        self.build_parametrized(lambda).map(Lambda::new)
    }

    /// Build a pi type
    pub fn build_pi(&mut self, pi: &PiExpr<'a>) -> Result<Pi, Error<'a>> {
        let result = self.build_parametrized(pi)?;
        result
            .try_into_value()
            .map_err(|_| Error::Message("Pi type must parametrize a valid type"))
            .map(Pi::new)
    }

    /// Build a parametrized value
    pub fn build_parametrized(
        &mut self,
        param: &ParametrizedExpr<'a>,
    ) -> Result<Parametrized<ValId>, Error<'a>> {
        self.push_args(&param.args)?;
        let result = self.build_expr(&param.result);
        let region = self
            .pop_region()
            .expect("`push_args` should always push a region to the stack!");
        Parametrized::try_new(result?, region)
            .map_err(|err| Error::ValueError("Invalid parametrized value", err))
    }

    /// Parse an expression, and return it
    pub fn parse_expr(&mut self, expr: &'a str) -> Result<(&'a str, ValId), Error<'a>> {
        let (rest, expr) = parse_expr(expr).map_err(|_| Error::ParseError(expr))?;
        self.build_expr(&expr).map(|value| (rest, value))
    }

    /// Parse and process a statement
    pub fn parse_statement(&mut self, statement: &'a str) -> Result<&'a str, Error<'a>> {
        let (rest, statement) =
            parse_statement(statement).map_err(|_| Error::ParseError(statement))?;
        self.build_statement(&statement)?;
        Ok(rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn units_build_properly() {
        let mut builder = Builder::<&str>::new();
        assert_eq!(
            builder.parse_expr("#unit").unwrap(),
            ("", ValId::from(Unit))
        );
        assert_eq!(builder.parse_expr("()").unwrap(), ("", ValId::from(())));
        assert_eq!(builder.parse_expr("[]").unwrap(), ("", ValId::from(())));
    }
    #[test]
    fn bad_indices_fail_properly() {
        let mut builder = Builder::<&str>::new();
        // Unit cannot be indexed
        assert!(builder.parse_expr("[] #ix(1)[0]").is_err());
        // Unit has no second index
        assert!(builder.parse_expr("#ix(1)[1]").is_err());
        // Empty type has no index
        assert!(builder.parse_expr("#ix(0)[0]").is_err());
        // Index out of bounds
        assert!(builder.parse_expr("[#true #false] #ix(2)[2]").is_err());
        // Index type out of bounds
        assert!(builder.parse_expr("[#true #false] #ix(3)[2]").is_err());
    }
}
