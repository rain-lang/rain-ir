/*!
An AST for `rain` programs
*/
use super::{parse_ident, PATH_SEP, SEXPR_CLOSE, SEXPR_OPEN, TUPLE_CLOSE, TUPLE_OPEN};
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
            return Ok(())
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
#[derive(PartialEq, Eq, Hash)]
pub struct Member<'a> {
    /// The value whose member is being accessed
    pub base: Box<Expr<'a>>,
    /// The path to the member being accessed
    pub path: Path<'a>
}

impl Display for Member<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}{}", self.base, self.path)
    }
}

debug_from_display!(Member<'_>);

/// An S-expression
#[derive(Eq, PartialEq, Hash, Default)]
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
#[derive(Eq, PartialEq, Hash, Default)]
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

/// A `rain` expression
#[derive(PartialEq, Eq, Hash)]
pub enum Expr<'a> {
    /// An identifier
    Ident(Ident<'a>),
    /// An S-expression
    Sexpr(Sexpr<'a>),
    /// A tuple
    Tuple(Tuple<'a>),
    /// A member access
    Member(Member<'a>)
}

impl Display for Expr<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Expr::Ident(i) => Display::fmt(i, fmt),
            Expr::Sexpr(s) => Display::fmt(s, fmt),
            Expr::Tuple(t) => Display::fmt(t, fmt),
            Expr::Member(m) => Display::fmt(m, fmt)
        }
    }
}

debug_from_display!(Expr<'_>);
