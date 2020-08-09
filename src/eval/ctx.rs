/*!
A `rain` evaluation context
*/

use super::Error;
use super::Substitute;
use crate::lifetime::{Color, Lifetime};
use crate::region::{Region, Regional};
use crate::typing::Typed;
use crate::value::{ValId, Value};
use fxhash::FxBuildHasher;
use im_rc::{HashMap, Vector};
use std::iter::Iterator;

/// A `rain` evaluation context
#[derive(Debug, Clone, PartialEq)]
pub struct EvalCtx {
    /// The cache for evaluated values
    eval_cache: HashMap<ValId, ValId, FxBuildHasher>,
    /// The color map
    color_map: HashMap<Color, Lifetime, FxBuildHasher>,
    /// The cache for lifetime substitutions
    lt_cache: HashMap<Lifetime, Lifetime, FxBuildHasher>,
    /// The parents of this context
    parents: Vector<EvalCtx>,
    /// The root depth of this evaluation context
    /// Every context below this depth is assumed to be empty
    root_depth: usize,
    /// This context's current region
    curr_region: Option<Region>,
}

impl EvalCtx {
    /// Create a new, empty evaluation context *within* a given region
    #[inline]
    pub fn new(root_depth: usize) -> EvalCtx {
        EvalCtx {
            eval_cache: HashMap::default(),
            color_map: HashMap::default(),
            lt_cache: HashMap::default(),
            parents: Vector::new(),
            root_depth,
            curr_region: None,
        }
    }
    /// Get whether this evaluation context is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.eval_cache.is_empty() && self.color_map.is_empty() && self.lt_cache.is_empty()
    }
    /// Get the root depth
    #[inline]
    pub fn root_depth(&self) -> usize {
        self.root_depth
    }
    /// Get the current evaluation depth
    #[inline]
    pub fn depth(&self) -> usize {
        let depth = self.root_depth + self.parents.len();
        if self.curr_region.is_some() {
            debug_assert_eq!(depth, self.curr_region.depth());
        }
        depth
    }
    /// Push a new (empty) scope onto this evaluation context
    #[inline]
    pub fn push(&mut self) {
        if self.is_empty() {
            self.root_depth += 1
        } else {
            let old_self = self.clone();
            self.parents.push_back(old_self)
        }
    }
    /// Get the parent of this evaluation context, if any
    #[inline]
    pub fn parent(&self) -> Option<&EvalCtx> {
        self.parents.last()
    }
    /// Clear this evaluation context, setting the root depth to the current evaluation depth
    #[inline]
    pub fn clear(&mut self) {
        self.root_depth = self.depth();
        self.parents.clear();
        self.eval_cache.clear();
        self.color_map.clear();
        self.lt_cache.clear();
    }
    /// Get this evaluation context at a given depth
    ///
    /// Return `None` if the evaluation context at that depth is null or the depth is out of bounds
    #[inline]
    pub fn at_depth(&self, depth: usize) -> Option<&EvalCtx> {
        if depth < self.root_depth {
            return None;
        }
        let ix = depth - self.root_depth;
        if ix == self.parents.len() {
            return Some(self);
        }
        self.parents.get(ix)
    }
    /// Send this evaluation context to a given depth
    #[inline]
    pub fn send_to_depth(&mut self, depth: usize) {
        // No-op on equality
        if depth == self.depth() {
            return;
        }
        if let Some(ctx) = self.at_depth(depth).cloned() {
            // Restore previous state
            *self = ctx
        } else if depth > self.depth() {
            // Push on new state
            while depth > self.depth() {
                self.push()
            }
        } else {
            // Clear caches, send root pointer backwards
            self.clear();
            self.root_depth = depth;
        }
    }
    /// Pop a level off this evaluation context
    #[inline]
    pub fn pop(&mut self) {
        if let Some(last) = self.parents.last().cloned() {
            // Restore previous state
            *self = last
        } else {
            // Clear caches, send root pointer backwards
            self.clear();
            self.root_depth = self.root_depth.saturating_sub(1);
        }
    }
    /// Check whether this is a pre-checked context
    #[inline]
    pub fn is_checked(&self) -> bool {
        //TODO
        false
    }
    /// Register a substitution in the given scope.
    ///
    /// If `check_ty` is true, perform a type check and return an error on failure
    /// If `check_region` is true, perform a region check and return an error if the value is defined in too shallow or too deep of a region.
    /// It is logic error to invalidate region requirements, though this may be done *internally* for more efficient implementations
    #[inline]
    pub fn substitute_impl(
        &mut self,
        lhs: ValId,
        rhs: ValId,
        check_ty: bool,
        check_region: bool,
    ) -> Result<(), Error> {
        if check_ty && lhs.ty() != rhs.ty() {
            return Err(Error::TypeMismatch);
        }
        if check_region {
            //TODO: region check
        }
        self.eval_cache.insert(lhs, rhs);
        //TODO: lifetime substitutions
        Ok(())
    }
    /// Register a substitution in the given scope, always checking types and region validities
    #[inline]
    pub fn substitute(&mut self, lhs: ValId, rhs: ValId) -> Result<(), Error> {
        self.substitute_impl(lhs, rhs, true, true)
    }
    /// Register substitutes values for each value in a region, creating a new scope if necessary
    ///
    /// Return an error on a type/lifetime mismatch.
    /// If `inline` is true, create a new region with any leftover parameters, and return it if made
    /// If `inline` is false, return an error in this case.
    #[inline]
    pub fn substitute_region<I>(
        &mut self,
        region: &Region,
        mut values: I,
        inline: bool,
    ) -> Result<Option<Region>, Error>
    where
        I: Iterator<Item = ValId>,
    {
        // Get the LCR, returning an error on incompatible regions
        let lcr = region.lcr(&self.curr_region)?;
        let lcr_depth = lcr.depth();
        // Check if the current region is not the LCR
        if lcr != self.curr_region.region() {
            // Get the evaluation context at the LCR, if any, and substitute within it
            let mut at_lcr = self
                .at_depth(lcr_depth - 1)
                .cloned()
                .unwrap_or_else(|| EvalCtx::new(lcr_depth - 1));
            debug_assert_eq!(at_lcr.curr_region.region(), lcr.ancestor(lcr_depth - 1));
            let result = at_lcr.substitute_region(region, values, inline)?;
            *self = at_lcr;
            return Ok(result);
        }
        let region_depth = region.depth();
        struct OldCaches {
            eval_cache: HashMap<ValId, ValId, FxBuildHasher>,
            lt_cache: HashMap<Lifetime, Lifetime, FxBuildHasher>,
            color_map: HashMap<Color, Lifetime, FxBuildHasher>,
        };
        let mut old_caches: Option<OldCaches> = None;
        let old_is_empty = self.is_empty();
        for param in region.borrow_params().map(Value::into_val) {
            if let Some(value) = values.next() {
                // Save old caches, if necessary
                if !old_is_empty && old_caches.is_none() {
                    old_caches = Some(OldCaches {
                        eval_cache: self.eval_cache.clone(),
                        lt_cache: self.lt_cache.clone(),
                        color_map: self.color_map.clone(),
                    });
                }
                // In case of error, restore old caches or clear caches if none
                if let Err(err) = self.substitute_impl(param.clone(), value, true, false) {
                    if let Some(old_caches) = old_caches {
                        self.eval_cache = old_caches.eval_cache;
                        self.lt_cache = old_caches.lt_cache;
                        self.color_map = old_caches.color_map;
                    } else {
                        self.eval_cache.clear();
                        self.lt_cache.clear();
                        self.color_map.clear();
                    }
                    return Err(err);
                }
            } else if inline {
                unimplemented!("Partial region substitution")
            } else {
                return Err(Error::NoInlineError);
            }
        }
        if let Some(old_caches) = old_caches {
            // Build up parent regions
            while self.root_depth < region_depth - 1 {
                self.parents.push_back(EvalCtx {
                    eval_cache: old_caches.eval_cache.clone(),
                    lt_cache: old_caches.lt_cache.clone(),
                    color_map: old_caches.color_map.clone(),
                    parents: self.parents.clone(),
                    curr_region: region.ancestor(self.root_depth).cloned_region(),
                    root_depth: self.root_depth,
                });
                self.root_depth += 1;
            }
            self.parents.push_back(EvalCtx {
                eval_cache: old_caches.eval_cache,
                lt_cache: old_caches.lt_cache,
                color_map: old_caches.color_map,
                parents: self.parents.clone(),
                curr_region: region.ancestor(self.root_depth).cloned_region(),
                root_depth: self.root_depth,
            });
            self.root_depth += 1;
        } else {
            // Just set the root depth directly
            self.root_depth = region_depth
        }
        // Set the current region
        self.curr_region = Some(region.clone());
        Ok(None)
    }
    /// Try to quickly evaluate a given value in the current scope. Return None on failure
    #[inline]
    pub fn try_evaluate(&self, value: &ValId) -> Option<ValId> {
        // Check if the value's depth is too shallow to have been touched by this context
        //TODO: proper region check?
        if value.depth() < self.root_depth {
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
        let root_depth = self.root_depth;
        let depth = self.depth();
        //NOTE: caching via try_evaluate is done in the implementation of Substitute for `ValId`
        let result = value.substitute(self);
        debug_assert_eq!(root_depth, self.root_depth);
        debug_assert_eq!(depth, self.depth());
        result
    }
    /// Evaulate a given lifetime. Return an error on evaluation failure.
    #[inline]
    pub fn evaluate_lt(&mut self, lifetime: &Lifetime) -> Result<Lifetime, Error> {
        // Ignore lifetimes below the root depth
        if lifetime.depth() < self.root_depth {
            return Ok(lifetime.clone());
        }
        // Check if the lifetime has been cached
        if let Some(lifetime) = self.lt_cache.get(lifetime) {
            return Ok(lifetime.clone());
        }
        // Attempt to color map the lifetime to the root depth
        let result = lifetime.color_map(
            |color| Some(self.color_map.get(color).unwrap_or(&Lifetime::STATIC)),
            //TODO: shallow borrow restriction?
            |value| self.eval_cache.get(value).cloned().ok_or(Error::UndefParam),
            self.root_depth,
        )?;
        self.lt_cache.insert(lifetime.clone(), result.clone());
        Ok(result)
    }
}
