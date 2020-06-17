/*!
Pattern matching branches

TODO: consider caching patterns
*/
use crate::function::pi::Pi;
use crate::primitive::logical::BOOL_TY;
use crate::region::{Parameter, Region};
use crate::value::{Error, VarRef};
use std::ops::Deref;
use triomphe::Arc;

/// A branch of a gamma node
#[derive(Debug, Clone, Hash, Eq)]
pub struct Pattern(pub Arc<PatternData>);

impl Match for Pattern {
    #[inline]
    fn try_match(&self, ty: VarRef<Pi>) -> Result<BranchArgs, Error> {
        self.deref().try_match(ty)
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
    /// A boolean pattern
    Bool(bool),
}

/// The parameters to a branch
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BranchArgs {
    /// This branch's region
    pub region: Region,
    /// This branch's parameters
    pub params: Vec<Parameter>,
}

/// A value which can perform pattern matching
pub trait Match {
    /// Try to match a pattern as a sub-branch of a given pi-type
    fn try_match(&self, ty: VarRef<Pi>) -> Result<BranchArgs, Error>;
}

impl Match for PatternData {
    #[inline]
    fn try_match(&self, ty: VarRef<Pi>) -> Result<BranchArgs, Error> {
        match self {
            PatternData::Any(a) => a.try_match(ty),
            PatternData::Bool(b) => b.try_match(ty),
        }
    }
}

/// A pattern which matches any argument vector, returning it unchanged
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Any;

impl Match for Any {
    fn try_match(&self, ty: VarRef<Pi>) -> Result<BranchArgs, Error> {
        Ok(BranchArgs {
            region: ty.def_region().clone(),
            params: ty.params().collect(),
        })
    }
}

impl Match for bool {
    fn try_match(&self, ty: VarRef<Pi>) -> Result<BranchArgs, Error> {
        let tdr = ty.def_region();
        // Filter for a single boolean argument
        if tdr.len() != 1 {
            return Err(Error::TupleLengthMismatch);
        }
        if &tdr[0] != BOOL_TY.deref() {
            return Err(Error::TypeMismatch);
        }
        Any.try_match(ty)
    }
}
