/*!
A `rain` evaluation context
*/

use super::Error;
use super::Substitute;
use crate::lifetime::{Color, Lifetime, Live};
use crate::region::{Region, Regional};
use crate::typing::Typed;
use crate::value::{ValId, Value};
use fxhash::FxBuildHasher;
use hayami::{SymbolMap, SymbolTable};
use smallvec::{smallvec, SmallVec};
use std::iter::Iterator;

/// A `rain` evaluation context
#[derive(Debug, Clone, PartialEq)]
pub struct EvalCtx {
    /// The cache for evaluated values
    eval_cache: SymbolTable<ValId, ValId, FxBuildHasher>,
    /// The color map
    color_map: SymbolTable<Color, Lifetime, FxBuildHasher>,
    /// The cache for lifetime substitutions
    lt_cache: SymbolTable<Lifetime, Lifetime, FxBuildHasher>,
    /// The minimum region depths at each scope level
    minimum_depths: SmallVec<[usize; 2]>,
}

impl EvalCtx {
    /// Create a new, empty evaluation context with a given capacity
    #[inline]
    pub fn with_capacity(e: usize, l: usize, c: usize) -> EvalCtx {
        EvalCtx {
            eval_cache: SymbolTable::with_capacity_and_hasher(e, FxBuildHasher::default()),
            color_map: SymbolTable::with_capacity_and_hasher(c, FxBuildHasher::default()),
            lt_cache: SymbolTable::with_capacity_and_hasher(l, FxBuildHasher::default()),
            minimum_depths: smallvec![std::usize::MAX],
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
        self.eval_cache.push();
        self.color_map.push();
        self.lt_cache.push();
        self.minimum_depths.push(self.minimum_depth());
    }
    /// Pop a scope from the evaluation context
    #[inline]
    pub fn pop(&mut self) {
        self.eval_cache.pop();
        self.color_map.pop();
        self.lt_cache.pop();
        self.minimum_depths.pop();
    }
    /// Get the current scope depth
    #[inline]
    pub fn scope_depth(&self) -> usize {
        let depth = self.minimum_depths.len() - 1;
        debug_assert_eq!(depth, self.eval_cache.depth());
        debug_assert_eq!(depth, self.lt_cache.depth());
        debug_assert_eq!(depth, self.color_map.depth());
        depth
    }
    /// Check whether this is a pre-checked context
    #[inline]
    pub fn is_checked(&self) -> bool {
        //TODO
        false
    }
    /// Register a substitution in the given scope. Return an error on a type/lifetime mismatch.    
    #[inline]
    pub fn substitute(&mut self, lhs: ValId, rhs: ValId, check: bool) -> Result<(), Error> {
        let llt = lhs.lifetime();
        if check {
            if !(llt >= rhs.lifetime()) {
                return Err(Error::LifetimeError);
            }
            if lhs.ty() != rhs.ty() {
                //TODO: subtyping
                return Err(Error::TypeMismatch);
            }
            let top = self.minimum_depths.last_mut().unwrap();
            *top = (*top).min(llt.depth());
        }
        self.eval_cache.insert(lhs, rhs);
        Ok(())
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
        for param in region.borrow_params().map(Value::into_val) {
            if let Some(value) = values.next() {
                self.substitute(param.clone(), value, check)?;
            } else if inline {
                unimplemented!("Partial region substitution")
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
        if value.depth() > self.minimum_depth() {
            return Some(value.clone());
        }
        // Check the cache
        if let Some(value) = self.eval_cache.get(value) {
            return Some(value.clone());
        }
        None
    }
    /// Evaluate a given value in the current scope. Return an error on evaluation failure.
    #[inline]
    pub fn evaluate(&mut self, value: &ValId) -> Result<ValId, Error> {
        let current_depth = self.minimum_depth();
        let scope_depth = self.scope_depth();
        //NOTE: caching via try_evaluate is done in the implementation of Substitute for `ValId`
        let result = value.substitute(self);
        debug_assert_eq!(self.minimum_depth(), current_depth);
        debug_assert_eq!(self.scope_depth(), scope_depth);
        result
    }
    /// Evaulate a given lifetime. Return an error on evaluation failure.
    #[inline]
    pub fn evaluate_lt(&mut self, lifetime: &Lifetime) -> Result<Lifetime, Error> {
        let current_depth = self.minimum_depth();
        // Ignore lifetimes out of the minimum depth
        if lifetime.depth() < current_depth {
            return Ok(lifetime.clone());
        }
        // Check if the lifetime has been cached
        if let Some(lifetime) = self.lt_cache.get(lifetime) {
            return Ok(lifetime.clone());
        }
        // Attempt to color map the lifetime
        let result = lifetime.color_map(
            |color| Some(self.color_map.get(color).unwrap_or(&Lifetime::STATIC)),
            current_depth,
        )?;
        self.lt_cache.insert(lifetime.clone(), result.clone());
        Ok(result)
    }
}
