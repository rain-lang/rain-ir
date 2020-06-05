/*!
A `rain` evaluation context
*/

use crate::util::symbol_table::SymbolTable;
use crate::value::{
    lifetime::{Live, Region},
    typing::Typed,
    ValId,
};
use fxhash::FxBuildHasher;
use std::hash::BuildHasher;
use std::iter::Iterator;

/// A `rain` evaluation context
#[derive(Debug, Clone, PartialEq)]
pub struct EvalCtx<S: BuildHasher = FxBuildHasher> {
    /// The cache for evaluated values
    cache: SymbolTable<ValId, ValId, S>,
}

impl<S: BuildHasher + Default> EvalCtx<S> {
    /// Create a new, empty evaluation context with a given capacity
    #[inline]
    pub fn with_capacity(n: usize) -> EvalCtx<S> {
        EvalCtx {
            cache: SymbolTable::with_capacity(n),
        }
    }
    /// Push a new (empty) scope onto the evaluation context
    #[inline]
    pub fn push(&mut self) {
        self.cache.push()
    }
    /// Pop a scope from the evaluation context
    #[inline]
    pub fn pop(&mut self) {
        self.cache.pop()
    }
    /// Register a substitution in the given scope. Return an error on a type/lifetime mismatch.
    /// 
    /// Return whether the substitution is already registred, in which case nothing happens.
    #[inline]
    pub fn substitute(&mut self, lhs: ValId, rhs: ValId, check: bool) -> Result<bool, ()> {
        if check {
            if lhs.ty() != rhs.ty() {
                //TODO: subtyping
                return Err(());
            }
            if !(lhs.lifetime() >= rhs.lifetime()) {
                return Err(());
            }
        }
        Ok(self.cache.try_def(lhs, rhs).map_err(|_| ()).is_err())
    }
    /// Register substitutes values for each value in a region.
    ///
    /// Return an error on a type/lifetime mismatch.
    /// Return how many region parameters were substituted, if any.
    #[inline]
    pub fn substitute_region<I>(
        &mut self,
        region: &Region,
        mut values: I,
        check: bool
    ) -> Result<usize, ()>
    where
        I: Iterator<Item = ValId>,
    {
        let mut sub = 0;
        for param in region.borrow_params().map(ValId::from) {
            if let Some(value) = values.next() {
                self.substitute(param, value, check)?;
                sub += 1;
            } else {
                break;
            }
        }
        Ok(sub)
    }
    /// Register substitutes values for each value in a region, in a new scope
    ///
    /// Return an error on a type/lifetime mismatch.
    /// Return how many region parameters were substituted, if any.
    /// On an error, undo all substitutions
    #[inline]
    pub fn push_region<I>(
        &mut self,
        region: &Region,
        values: I,
        check: bool
    ) -> Result<usize, ()>
    where
        I: Iterator<Item = ValId>,
    {
        self.push();
        self.substitute_region(region, values, check).map_err(|err| {
            self.pop();
            err
        })
    }
    /// Evaluate a given value in the current scope. Return an error on evaluation failure.
    #[inline]
    pub fn evaluate(&mut self, value: &ValId) -> Result<ValId, ()> {
        if let Some(value) = self.cache.get(value) {
            return Ok(value.clone())
        } else {
            unimplemented!()
        }
    }
}
