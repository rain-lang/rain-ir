/*!
An AST for `rain` programs
*/
use crate::{debug_from_display, quick_display};
use smallvec::SmallVec;
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};

/// An identifier
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ident<'a>(pub(super) &'a str);

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
pub struct Path<'a>(pub IdentVec<'a>);

impl Display for Path<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        if self.len() == 0 {
            // Edge case
            return write!(fmt, ".");
        }
        let mut first = true;
        for ident in self.iter() {
            write!(fmt, "{}{}", if first { "" } else { "." }, ident)?;
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

/// A `rain` expression
pub enum Expr<'a> {
    /// A path denoting a given `rain` value
    Path(Path<'a>),
}

impl Display for Expr<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Expr::Path(p) => Display::fmt(p, fmt),
        }
    }
}

debug_from_display!(Expr<'_>);
