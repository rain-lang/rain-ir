/*!
Pattern matching branches

TODO: consider caching patterns
*/
use crate::function::pi::Pi;
use crate::primitive::logical::{BOOL_TY, FALSE, TRUE};
use crate::typing::Typed;
use crate::value::{Error, TypeId, ValId, VarRef};
use itertools::Itertools;
use std::ops::Deref;
use triomphe::Arc;

/// A branch of a gamma node
#[derive(Debug, Clone, Hash, Eq)]
pub struct Pattern(pub Arc<PatternData>);

impl Match for Pattern {
    #[inline]
    fn try_match_ty(&self, ty: VarRef<Pi>) -> Result<MatchedTy, Error> {
        self.deref().try_match_ty(ty)
    }
    #[inline]
    fn try_match(&self, inp: &[ValId]) -> Result<Matched, Error> {
        self.deref().try_match(inp)
    }
}

impl Deref for Pattern {
    type Target = PatternData;
    #[inline]
    fn deref(&self) -> &PatternData {
        &self.0
    }
}

impl PartialEq for Pattern {
    #[inline]
    fn eq(&self, other: &Pattern) -> bool {
        let l = self.deref();
        let r = other.deref();
        std::ptr::eq(l, r) || l == r
    }
}

impl From<PatternData> for Pattern {
    fn from(pattern: PatternData) -> Pattern {
        Pattern(pattern.into())
    }
}

/// The data underlying a branch of a gamma node
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PatternData {
    /// The wildcard pattern
    Any(Any),
    /// The empty pattern
    Empty(Empty),
    /// A boolean pattern
    Bool(bool),
}

/// The match result types for a branch
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MatchedTy(pub Vec<TypeId>);

/// The match results for a branch
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Matched(pub Vec<ValId>);

/// A value which can perform pattern matching
pub trait Match {
    /// Try to match a pattern as a sub-branch of a given pi-type
    fn try_match_ty(&self, ty: VarRef<Pi>) -> Result<MatchedTy, Error>;
    /// Try to match a value according to a pattern
    fn try_match(&self, inp: &[ValId]) -> Result<Matched, Error>;
}

impl Match for PatternData {
    #[inline]
    fn try_match_ty(&self, ty: VarRef<Pi>) -> Result<MatchedTy, Error> {
        match self {
            PatternData::Any(a) => a.try_match_ty(ty),
            PatternData::Empty(e) => e.try_match_ty(ty),
            PatternData::Bool(b) => b.try_match_ty(ty),
        }
    }
    #[inline]
    fn try_match(&self, inp: &[ValId]) -> Result<Matched, Error> {
        match self {
            PatternData::Any(a) => a.try_match(inp),
            PatternData::Empty(e) => e.try_match(inp),
            PatternData::Bool(b) => b.try_match(inp),
        }
    }
}

/// A pattern which matches any argument vector, returning it unchanged
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Any;

impl Match for Any {
    fn try_match_ty(&self, ty: VarRef<Pi>) -> Result<MatchedTy, Error> {
        Ok(MatchedTy(
            ty.def_region().param_tys().iter().cloned().collect_vec(),
        ))
    }
    fn try_match(&self, ty: &[ValId]) -> Result<Matched, Error> {
        Ok(Matched(ty.to_vec()))
    }
}

impl Match for bool {
    fn try_match_ty(&self, ty: VarRef<Pi>) -> Result<MatchedTy, Error> {
        let tdr = ty.def_region();
        // Filter for a single boolean argument
        if tdr.len() != 1 {
            return Err(Error::TupleLengthMismatch);
        }
        if &tdr[0] != BOOL_TY.deref() {
            return Err(Error::TypeMismatch);
        }
        Any.try_match_ty(ty)
    }
    fn try_match(&self, ty: &[ValId]) -> Result<Matched, Error> {
        if ty.len() != 1 {
            return Err(Error::TupleLengthMismatch);
        }
        if ty[0].ty() != BOOL_TY.borrow_ty() {
            return Err(Error::TypeMismatch);
        }
        match self {
            true if &ty[0] == TRUE.deref() => Ok(Matched(vec![])),
            false if &ty[0] == FALSE.deref() => Ok(Matched(vec![])),
            _ => Err(Error::MatchFailure),
        }
    }
}

/// The empty pattern, which does not match any argument vector
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Empty;

impl Match for Empty {
    fn try_match_ty(&self, _ty: VarRef<Pi>) -> Result<MatchedTy, Error> {
        Ok(MatchedTy(vec![]))
    }
    fn try_match(&self, _inp: &[ValId]) -> Result<Matched, Error> {
        Ok(Matched(vec![]))
    }
}
