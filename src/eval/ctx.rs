/*!
A `rain` evaluation context
*/

use super::Error;
use super::Substitute;
use crate::lifetime::{Live, Region};
use crate::typing::Typed;
use crate::util::symbol_table::SymbolTable;
use crate::value::ValId;
use fxhash::FxBuildHasher;
use smallvec::{smallvec, SmallVec};
use std::iter::Iterator;

/// A `rain` evaluation context
#[derive(Debug, Clone, PartialEq)]
pub struct EvalCtx {
    /// The cache for evaluated values
    cache: SymbolTable<ValId, ValId, FxBuildHasher>,
    /// The minimum region depths at each scope level
    minimum_depths: SmallVec<[usize; 2]>,
}

impl EvalCtx {
    /// Create a new, empty evaluation context with a given capacity
    #[inline]
    pub fn with_capacity(n: usize) -> EvalCtx {
        EvalCtx {
            cache: SymbolTable::with_capacity(n),
            minimum_depths: smallvec![usize::MAX],
        }
    }
    /// Get the current minimum depth
    #[inline]
    pub fn minimum_depth(&self) -> usize {
        self.minimum_depths.last().copied().unwrap_or(usize::MAX)
    }
    /// Push a new (empty) scope onto the evaluation context
    #[inline]
    pub fn push(&mut self) {
        self.cache.push();
        self.minimum_depths.push(self.minimum_depth());
    }
    /// Pop a scope from the evaluation context
    #[inline]
    pub fn pop(&mut self) {
        self.cache.pop();
        self.minimum_depths.pop();
    }
    /// Check whether this is a pre-checked context
    #[inline]
    pub fn is_checked(&self) -> bool {
        //TODO
        false
    }
    /// Register a substitution in the given scope. Return an error on a type/lifetime mismatch.
    ///
    /// Return whether the substitution is already registred, in which case nothing happens.
    #[inline]
    pub fn substitute(&mut self, lhs: ValId, rhs: ValId, check: bool) -> Result<bool, Error> {
        let llt = lhs.lifetime();
        if check {
            if !(llt >= rhs.lifetime()) {
                return Err(Error::LifetimeError);
            }
            if lhs.ty() != rhs.ty() {
                //TODO: subtyping
                return Err(Error::TypeMismatch);
            }
            if let Some(top) = self.minimum_depths.last_mut() {
                *top = (*top).min(llt.depth());
            }
        }
        Ok(self.cache.try_def(lhs, rhs).map_err(|_| ()).is_err())
    }
    /// Register substitutes values for each value in a region.
    ///
    /// Return an error on a type/lifetime mismatch.
    /// If `inline` is true, create a new region with any leftover parameters, and return it if made
    /// If `inline` is false, return an error in this case.
    #[inline]
    pub fn substitute_region<I>(
        &mut self,
        region: &Region,
        mut values: I,
        check: bool,
        inline: bool,
    ) -> Result<Option<Region>, Error>
    where
        I: Iterator<Item = ValId>,
    {
        for param in region.borrow_params().map(ValId::from) {
            if let Some(value) = values.next() {
                self.substitute(param.clone(), value, check)?;
            } else if inline {
                //TODO: this
                break;
            } else {
                return Err(Error::NoInlineError);
            }
        }
        Ok(None)
    }
    /// Register substitutes values for each value in a region, in a new scope
    ///
    /// Return an error on a type/lifetime mismatch.
    /// If `inline` is true, create a new region with any leftover parameters, and return it if made
    /// If `inline` is false, return an error in this case.
    /// On an error, undo all substitutions
    #[inline]
    pub fn push_region<I>(
        &mut self,
        region: &Region,
        values: I,
        check: bool,
        inline: bool,
    ) -> Result<Option<Region>, Error>
    where
        I: Iterator<Item = ValId>,
    {
        self.push();
        let result = self
            .substitute_region(region, values, check, inline)
            .map_err(|err| {
                self.pop();
                err
            });
        if result == Err(Error::NoInlineError) {
            return Ok(None);
        }
        result
    }
    /// Try to quickly evaluate a given value in the current scope. Return None on failure
    #[inline]
    pub fn try_evaluate(&self, value: &ValId) -> Option<ValId> {
        // Check if the value's depth is too deep to have been touched by this context
        if value.lifetime().depth() > self.minimum_depth() {
            return Some(value.clone());
        }
        // Check the cache
        if let Some(value) = self.cache.get(value) {
            return Some(value.clone());
        }
        None
    }
    /// Evaluate a given value in the current scope. Return an error on evaluation failure.
    #[inline]
    pub fn evaluate(&mut self, value: &ValId) -> Result<ValId, Error> {
        // Substitute the value
        // (TODO: depth first search to avoid stack overflow, maybe...)
        value.substitute(self)
    }
}
