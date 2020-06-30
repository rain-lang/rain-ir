/*!
Pattern matching branches

TODO: consider caching patterns
*/
use crate::primitive::logical::{BOOL_TY, FALSE, TRUE};
use crate::typing::Typed;
use crate::value::{Error, TypeId, ValId};
use elysees::Arc;
use std::ops::Deref;

/// A branch of a gamma node
#[derive(Debug, Clone, Hash, Eq)]
pub struct Pattern(pub Arc<PatternData>);

impl Pattern {
    /// Create the empty pattern
    pub fn empty() -> Pattern {
        Empty.into()
    }
    /// Get the disjunction of two patterns
    pub fn disjunction(&self, other: &Pattern) -> Pattern {
        let mut m = self.clone();
        m.take_disjunction(other);
        m
    }
    /// Make this pattern into the disjunction of two patterns
    pub fn take_disjunction(&mut self, other: &Pattern) {
        match other.deref() {
            PatternData::Any(_) => {
                if self != other {
                    *self = other.clone()
                }
            }
            PatternData::Empty(_) => {}
            PatternData::Bool(l) => match (*self).deref() {
                PatternData::Bool(r) => {
                    if l != r {
                        *self = Pattern::from(Any)
                    }
                }
                PatternData::Any(_) => {}
                PatternData::Empty(_) => *self = other.clone(),
            },
        }
    }
    /// Check whether a pattern is complete
    pub fn is_complete(&self) -> bool {
        match self.deref() {
            PatternData::Any(_) => true,
            _ => false,
        }
    }
}

impl Match for Pattern {
    #[inline]
    fn matches(&self, args: &[TypeId], outputs: &[TypeId]) -> Result<(), Error> {
        self.deref().matches(args, outputs)
    }
    #[inline]
    fn try_get_outputs(&self, ty: &[TypeId]) -> Result<MatchedTy, Error> {
        self.deref().try_get_outputs(ty)
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
    /// Return Ok(()) if a pattern can produce a given output type vector from a given input type vector
    fn matches(&self, args: &[TypeId], outputs: &[TypeId]) -> Result<(), Error>;
    /// Attempt to obtain an argument vector compatible with the given pi type's argument vector
    ///
    /// Satisfies the invariant that, should this call succeed, letting `outputs` be the resulting type vector
    /// and `args` be the pi type's argument vector, `self.matches(args, outputs) == Ok(())`.
    /// Note this may fail, especially in the case where e.g. there are two such possible vectors.
    fn try_get_outputs(&self, ty: &[TypeId]) -> Result<MatchedTy, Error>;
    /// Try to match a value according to a pattern
    fn try_match(&self, inp: &[ValId]) -> Result<Matched, Error>;
}

impl Match for PatternData {
    #[inline]
    fn matches(&self, args: &[TypeId], outputs: &[TypeId]) -> Result<(), Error> {
        match self {
            PatternData::Any(a) => a.matches(args, outputs),
            PatternData::Empty(e) => e.matches(args, outputs),
            PatternData::Bool(b) => b.matches(args, outputs),
        }
    }
    #[inline]
    fn try_get_outputs(&self, ty: &[TypeId]) -> Result<MatchedTy, Error> {
        match self {
            PatternData::Any(a) => a.try_get_outputs(ty),
            PatternData::Empty(e) => e.try_get_outputs(ty),
            PatternData::Bool(b) => b.try_get_outputs(ty),
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
    fn matches(&self, args: &[TypeId], outputs: &[TypeId]) -> Result<(), Error> {
        if args == outputs {
            //TODO: subtyping, etc.
            return Ok(());
        } else {
            return Err(Error::TypeMismatch);
        }
    }
    fn try_get_outputs(&self, tys: &[TypeId]) -> Result<MatchedTy, Error> {
        Ok(MatchedTy(tys.into()))
    }
    fn try_match(&self, ty: &[ValId]) -> Result<Matched, Error> {
        Ok(Matched(ty.to_vec()))
    }
}

impl From<Any> for PatternData {
    #[inline]
    fn from(a: Any) -> PatternData {
        PatternData::Any(a)
    }
}

impl From<Any> for Pattern {
    #[inline]
    fn from(a: Any) -> Pattern {
        PatternData::Any(a).into()
    }
}

impl Match for bool {
    fn matches(&self, args: &[TypeId], outputs: &[TypeId]) -> Result<(), Error> {
        if args.len() != 1 {
            return Err(Error::TupleLengthMismatch);
        }
        if &args[0] != BOOL_TY.deref() {
            return Err(Error::TypeMismatch);
        }
        if outputs.len() != 0 {
            return Err(Error::TupleLengthMismatch);
        }
        return Ok(());
    }
    fn try_get_outputs(&self, tys: &[TypeId]) -> Result<MatchedTy, Error> {
        // Filter for a single boolean argument
        if tys.len() != 1 {
            return Err(Error::TupleLengthMismatch);
        }
        if &tys[0] != BOOL_TY.deref() {
            return Err(Error::TypeMismatch);
        }
        Ok(MatchedTy(vec![]))
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

impl From<bool> for PatternData {
    #[inline]
    fn from(b: bool) -> PatternData {
        PatternData::Bool(b)
    }
}

impl From<bool> for Pattern {
    #[inline]
    fn from(b: bool) -> Pattern {
        PatternData::Bool(b).into()
    }
}

/// The empty pattern, which does not match any argument vector
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Empty;

impl Match for Empty {
    fn matches(&self, _args: &[TypeId], _outputs: &[TypeId]) -> Result<(), Error> {
        // `Empty` matches any input/output pair since the actual match always fails
        return Ok(());
    }
    fn try_get_outputs(&self, _ty: &[TypeId]) -> Result<MatchedTy, Error> {
        Ok(MatchedTy(vec![]))
    }
    fn try_match(&self, _inp: &[ValId]) -> Result<Matched, Error> {
        Ok(Matched(vec![]))
    }
}

impl From<Empty> for PatternData {
    #[inline]
    fn from(e: Empty) -> PatternData {
        PatternData::Empty(e)
    }
}

impl From<Empty> for Pattern {
    #[inline]
    fn from(e: Empty) -> Pattern {
        PatternData::Empty(e).into()
    }
}
