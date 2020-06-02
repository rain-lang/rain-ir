/*!
An AST for `rain` programs
*/
use super::{parse_ident, parse_u128};
use crate::prettyprinter::tokens::*;
use crate::value::primitive::{finite::Finite, logical::Bool};
use crate::{debug_from_display, quick_display};
use smallvec::SmallVec;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};

/// An identifier
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ident<'a>(pub(super) &'a str);

impl<'a> TryFrom<&'a str> for Ident<'a> {
    type Error = (); //TODO: think about this
    fn try_from(s: &'a str) -> Result<Ident<'a>, ()> {
        match parse_ident(s) {
            Ok(("", s)) => Ok(s),
            _ => Err(()),
        }
    }
}

quick_display!(Ident<'_>, id, fmt => Display::fmt(id.0, fmt));
debug_from_display!(Ident<'_>);

impl<'a> Ident<'a> {
    /// Get the string underlying this identifier
    #[inline]
    pub fn get_str(&self) -> &'a str {
        self.0
    }
    /// Get this identifier as a symbol string. Return `None` for the null symbol.
    #[inline]
    pub fn get_sym(&self) -> Result<Option<&'a str>, Ident<'a>> {
        if self.get_str() == NULL_SYMBOL {
            Ok(None)
        } else {
            //TODO: number checking, etc.
            Ok(Some(self.get_str()))
        }
    }
    /// Try to convert this identifier to a `u128`. Return it on failure
    #[inline]
    pub fn get_u128(&self) -> Result<u128, Ident<'a>> {
        match parse_u128(self.0) {
            Ok(("", r)) => Ok(r),
            _ => Err(*self)
        }
    }
}

impl<'a> Deref for Ident<'a> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.0
    }
}

/// The size of a small path
pub const SMALL_PATH: usize = 1;

/// A small vector of identifiers
pub type IdentVec<'a> = SmallVec<[Ident<'a>; SMALL_PATH]>;

/// A path
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Path<'a>(pub IdentVec<'a>);

impl<'a> Path<'a> {
    /// Create a new empty path
    pub fn empty() -> Path<'a> {
        Path(IdentVec::new())
    }
}

impl Display for Path<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        if self.len() == 0 {
            // Edge case (temporary)
            return Ok(());
        }
        for ident in self.iter() {
            write!(fmt, "{}{}", PATH_SEP, ident)?;
        }
        Ok(())
    }
}

debug_from_display!(Path<'_>);

impl<'a> Deref for Path<'a> {
    type Target = IdentVec<'a>;
    #[inline]
    fn deref(&self) -> &IdentVec<'a> {
        &self.0
    }
}

impl<'a> DerefMut for Path<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut IdentVec<'a> {
        &mut self.0
    }
}

/// A member access expression
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Member<'a> {
    /// The value whose member is being accessed
    pub base: Box<Expr<'a>>,
    /// The path to the member being accessed
    pub path: Path<'a>,
}

impl Display for Member<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}{}", self.base, self.path)
    }
}

debug_from_display!(Member<'_>);

/// An S-expression
#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct Sexpr<'a>(pub Vec<Expr<'a>>);

impl<'a> Sexpr<'a> {
    /// Create a new empty S-expression
    pub fn unit() -> Sexpr<'a> {
        Sexpr(Vec::new())
    }
}

impl<'a> Deref for Sexpr<'a> {
    type Target = Vec<Expr<'a>>;
    #[inline]
    fn deref(&self) -> &Vec<Expr<'a>> {
        &self.0
    }
}

impl<'a> DerefMut for Sexpr<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Vec<Expr<'a>> {
        &mut self.0
    }
}

impl Display for Sexpr<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", SEXPR_OPEN)?;
        let mut first = true;
        for expr in self.iter() {
            write!(fmt, "{}{}", if first { "" } else { " " }, expr)?;
            first = false;
        }
        write!(fmt, "{}", SEXPR_CLOSE)
    }
}

debug_from_display!(Sexpr<'_>);

/// A tuple
#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct Tuple<'a>(pub Vec<Expr<'a>>);

impl<'a> Tuple<'a> {
    /// Create a new empty tuple
    pub fn unit() -> Tuple<'a> {
        Tuple(Vec::new())
    }
}

impl<'a> Deref for Tuple<'a> {
    type Target = Vec<Expr<'a>>;
    #[inline]
    fn deref(&self) -> &Vec<Expr<'a>> {
        &self.0
    }
}

impl<'a> DerefMut for Tuple<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Vec<Expr<'a>> {
        &mut self.0
    }
}

impl Display for Tuple<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", TUPLE_OPEN)?;
        let mut first = true;
        for expr in self.iter() {
            write!(fmt, "{}{}", if first { "" } else { " " }, expr)?;
            first = false;
        }
        write!(fmt, "{}", TUPLE_CLOSE)
    }
}

debug_from_display!(Tuple<'_>);

/// A tuple
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TypeOf<'a>(pub Box<Expr<'a>>);

impl<'a> TypeOf<'a> {
    /// Create a new empty tuple
    pub fn unit() -> Tuple<'a> {
        Tuple(Vec::new())
    }
}

impl Display for TypeOf<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}({})", KEYWORD_TYPEOF, self.0)
    }
}

debug_from_display!(TypeOf<'_>);

/// An index into a finite type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Index {
    /// The type of this index
    pub ty: Option<Finite>,
    /// The index
    pub ix: u128,
}

debug_from_display!(Index);

impl Display for Index {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        if let Some(ty) = self.ty {
            write!(fmt, "{}({})[{}]", KEYWORD_IX, ty, self.ix)
        } else {
            write!(fmt, "{}[{}]", KEYWORD_IX, self.ix)
        }
    }
}

/// A `rain` expression
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Expr<'a> {
    /// An identifier
    Ident(Ident<'a>),
    /// An S-expression
    Sexpr(Sexpr<'a>),
    /// A tuple
    Tuple(Tuple<'a>),
    /// A member access
    Member(Member<'a>),
    /// A boolean
    Bool(bool),
    /// The boolean type
    BoolTy(Bool),
    /// A typeof expression
    TypeOf(TypeOf<'a>),
    /// A finite type
    Finite(Finite),
    /// An index into a finite type
    Index(Index),
}

impl Display for Expr<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Expr::Ident(i) => Display::fmt(i, fmt),
            Expr::Sexpr(s) => Display::fmt(s, fmt),
            Expr::Tuple(t) => Display::fmt(t, fmt),
            Expr::Member(m) => Display::fmt(m, fmt),
            Expr::Bool(b) => match b {
                true => write!(fmt, "{}", KEYWORD_TRUE),
                false => write!(fmt, "{}", KEYWORD_FALSE),
            },
            Expr::BoolTy(b) => Display::fmt(b, fmt),
            Expr::TypeOf(t) => Display::fmt(t, fmt),
            Expr::Finite(f) => Display::fmt(f, fmt),
            Expr::Index(i) => Display::fmt(i, fmt),
        }
    }
}

debug_from_display!(Expr<'_>);

/// A pattern to assign to
#[derive(Clone, Eq, PartialEq)]
pub enum Pattern<'a> {
    /// A simple assignment to a variable
    Simple(Simple<'a>),
    /// A tuple-destructure assignment
    Detuple(Detuple<'a>),
}

impl Display for Pattern<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Pattern::Simple(s) => Display::fmt(s, fmt),
            Pattern::Detuple(d) => Display::fmt(d, fmt),
        }
    }
}

debug_from_display!(Pattern<'_>);

/// A simple assignment to a variable
#[derive(Clone, Eq, PartialEq)]
pub struct Simple<'a> {
    /// The name of the variable being assigned to
    pub var: Ident<'a>,
    /// A type-bound on the variable being assigned to, if any
    pub ty: Option<Expr<'a>>,
}

impl Display for Simple<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.var)?;
        if let Some(ty) = self.ty.as_ref() {
            write!(fmt, ": {}", ty)?
        }
        Ok(())
    }
}

debug_from_display!(Simple<'_>);

/// A tuple-destructure assignment pattern
#[derive(Clone, Eq, PartialEq)]
pub struct Detuple<'a>(pub Vec<Pattern<'a>>);

impl Display for Detuple<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", TUPLE_OPEN)?;
        let mut first = true;
        for pattern in self.0.iter() {
            write!(fmt, "{}{}", if first { "" } else { " " }, pattern)?;
            first = false;
        }
        write!(fmt, "{}", TUPLE_CLOSE)
    }
}

debug_from_display!(Detuple<'_>);

/// A let-statement
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Let<'a> {
    /// The pattern being assigned to
    pub lhs: Pattern<'a>,
    /// The value being assigned to the pattern
    pub rhs: Expr<'a>,
}

impl Display for Let<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{} {} {} {};", KEYWORD_LET, self.lhs, ASSIGN, self.rhs)
    }
}
