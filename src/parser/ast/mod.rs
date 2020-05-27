/*!
An AST for `rain` programs
*/
use super::{ident, PATH_SEP};
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
        match ident(s) {
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
            // Edge case
            return write!(fmt, "{}", PATH_SEP);
        }
        let mut first = true;
        for ident in self.iter() {
            write!(fmt, "{}{}", if first { "" } else { PATH_SEP }, ident)?;
            first = false;
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
        write!(fmt, "(")?;
        let mut first = true;
        for expr in self.iter() {
            write!(fmt, "{}{}", if first { "" } else { " " }, expr)?;
            first = false;
        }
        write!(fmt, ")")
    }
}

debug_from_display!(Sexpr<'_>);

/// A `rain` expression
#[derive(PartialEq, Eq, Hash)]
pub enum Expr<'a> {
    /// A path denoting a given `rain` value
    Path(Path<'a>),
    /// An S-expression
    Sexpr(Sexpr<'a>),
}

impl Display for Expr<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Expr::Path(p) => Display::fmt(p, fmt),
            Expr::Sexpr(s) => Display::fmt(s, fmt),
        }
    }
}

debug_from_display!(Expr<'_>);
