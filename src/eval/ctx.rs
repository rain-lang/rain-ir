/*!
A `rain` evaluation context
*/

use super::Error;
use super::Substitute;
use crate::region::{Region, Regional};
use crate::typing::{Type, Typed};
use crate::value::{ValId, Value};
use fxhash::FxBuildHasher;
use im_rc::hashmap::Entry;
use im_rc::{HashMap, Vector};
use std::cmp::Ordering;
use std::iter::Iterator;
use std::ops::Deref;
use Ordering::*;

/// A `rain` evaluation context
#[derive(Debug, Clone, PartialEq)]
pub struct EvalCtx {
    /// The cache for evaluated values
    eval_cache: HashMap<ValId, ValId, FxBuildHasher>,
    /// The parents of this context
    parents: Vector<EvalCtx>,
    /// The current domain region of this evaluation context.
    /// Anything deeper than this region should just get transported down.
    domain_region: Region,
    /// The current target region of this evaluation context.
    /// Any substitution made must lie within this target region
    target_region: Region,
    /// The current root depth of this evaluation context.
    /// Anything shallower than this should just be ignored
    /// This must *always* be less than or equal to the depth of the current region
    root_depth: usize,
}

impl Default for EvalCtx {
    #[inline]
    fn default() -> EvalCtx {
        EvalCtx::new()
    }
}

impl EvalCtx {
    /// Create a new, empty evaluation context *within* a given region
    #[inline]
    pub fn new() -> EvalCtx {
        EvalCtx {
            eval_cache: HashMap::default(),
            parents: Vector::new(),
            root_depth: 0,
            domain_region: Region::NULL,
            target_region: Region::NULL,
        }
    }
    /// Get whether this evaluation context is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.eval_cache.is_empty()
    }
    /// Get the root depth
    #[inline]
    pub fn root_depth(&self) -> usize {
        self.root_depth
    }
    /// Get the current evaluation depth
    #[inline]
    pub fn depth(&self) -> usize {
        self.domain_region.depth()
    }
    /// Get the parent of this evaluation context, if any
    #[inline]
    pub fn parent(&self) -> Option<&EvalCtx> {
        self.parents.last()
    }
    /// Pop the most recently defined region from this evaluation context, if any
    #[inline]
    pub fn pop(&mut self) {
        if let Some(parent) = self.parent().cloned() {
            *self = parent
        } else {
            self.clear()
        }
    }
    /// Clear this evaluation context
    #[inline]
    pub fn clear(&mut self) {
        self.root_depth = 0;
        self.domain_region = Region::NULL;
        self.target_region = Region::NULL;
        self.parents.clear();
        self.eval_cache.clear();
    }
    /// Register a substitution in the given scope. Return the re-defined value, if any.
    ///
    /// We pass in the following flags
    /// - `check_ty`: if this is `true`, perform a type check. If false, assume the type is correct (it is a logic error if it is not).
    /// - `update_target`: if this is `true`, potentially make the target region deeper as required by the `rhs`
    /// - `check_target`: if this is `true`, checks the `rhs` lies within the target region. This happens pre-update in the case of updating targets
    /// - `check_domain`: if this is `true`, checks that the `lhs` lies within the current region *and* has depth greater than the root depth
    /// - `allow_redef`: if this is `true`, allow re-definition. If not, return an error
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn substitute_impl(
        &mut self,
        lhs: ValId,
        rhs: ValId,
        check_ty: bool,
        update_target: bool,
        check_target: bool,
        check_domain: bool,
        allow_redef: bool,
    ) -> Result<Option<ValId>, Error> {
        // Typecheck
        if check_ty && lhs != rhs {
            let lhs_sub_ty = lhs.ty().substitute_ty(self)?;
            if lhs_sub_ty != rhs.ty() {
                return Err(Error::TypeMismatch);
            }
        }

        // Target check/update
        if update_target || check_target {
            let rhs_region = rhs.region();
            match rhs_region.partial_cmp(&self.target_region) {
                None => return Err(Error::IncomparableRegions),
                Some(Greater) => {
                    if update_target {
                        self.target_region = rhs_region.clone_region();
                    } else {
                        return Err(Error::DeepSub);
                    }
                }
                _ => {}
            }
        } else {
            debug_assert!(
                rhs.region() <= self.target_region,
                "Invalid release-unchecked substitution: RHS NOT IN TARGET
RHS = {}
LHS = {}
ROOT_DEPTH = {}
RHS_REGION(depth = {}) <=> TARGET_REGION(depth = {}) = {:?}",
                rhs,
                lhs,
                self.root_depth(),
                rhs.depth(),
                self.target_region.depth(),
                rhs.region().partial_cmp(&self.target_region)
            );
        }

        // Region check
        if check_domain {
            let lhs_region = lhs.region();
            match lhs_region.partial_cmp(&self.domain_region) {
                None => return Err(Error::IncomparableRegions),
                Some(Greater) => return Err(Error::DeepSub),
                Some(Less) if lhs_region.depth() < self.root_depth() => {
                    return Err(Error::ShallowSub);
                }
                _ => {}
            }
        } else {
            debug_assert!(
                lhs.region() <= self.domain_region,
                "Invalid release-unchecked substitution: LHS NOT IN DOMAIN
RHS = {}
LHS = {}
ROOT_DEPTH = {}
LHS_REGION(depth = {}) <=> DOMAIN_REGION(depth = {}) = {:?}",
                rhs,
                lhs,
                self.root_depth(),
                lhs.depth(),
                self.domain_region.depth(),
                lhs.region().partial_cmp(&self.domain_region)
            );
            debug_assert!(
                lhs.depth() >= self.root_depth,
                "Invalid release-unchecked substitution: SHALLOW LHS SUBSTITUTION
RHS = {}
LHS = {}
ROOT_DEPTH = {}
LHS_REGION(depth = {}) <=> DOMAIN_REGION(depth = {}) = {:?}",
                rhs,
                lhs,
                self.root_depth(),
                lhs.depth(),
                self.domain_region.depth(),
                lhs.region().partial_cmp(&self.domain_region)
            );
        }

        // Evaluation cache insertion
        match self.eval_cache.entry(lhs) {
            Entry::Vacant(v) => {
                v.insert(rhs);
                Ok(None)
            }
            Entry::Occupied(mut o) => {
                if *o.get() == rhs {
                    Ok(None)
                } else if allow_redef {
                    Ok(Some(o.insert(rhs)))
                } else {
                    Err(Error::InvalidRedef)
                }
            }
        }
    }
    /// Register a substitution in the given scope, not checking anything!
    #[inline]
    pub fn substitute_unchecked(&mut self, lhs: ValId, rhs: ValId) -> Result<Option<ValId>, Error> {
        self.substitute_impl(lhs, rhs, false, false, false, false, false)
    }
    /// Register a substitution in the given scope, checking everything
    #[inline]
    pub fn substitute(&mut self, lhs: ValId, rhs: ValId) -> Result<(), Error> {
        self.substitute_impl(lhs, rhs, true, false, true, true, false)
            .map(|_| ())
    }
    /// The body of region substitution. Potentially leaves this context in an invalid state on error.
    fn substitute_region_body<I>(
        &mut self,
        region: &Region,
        mut values: I,
        inline: bool,
    ) -> Result<Option<Region>, Error>
    where
        I: Iterator<Item = ValId>,
    {
        let mut inline_params = None;
        for (ix, param) in region.params().enumerate() {
            if let Some(value) = values.next() {
                self.substitute_impl(param.into_val(), value, true, true, false, false, false)?;
            } else if inline {
                let inline_param = if let Some(inline_params) = &mut inline_params {
                    inline_params
                } else {
                    let new_target_region = Region::with(
                        region.param_tys().deref()[ix..]
                            .iter()
                            .map(|ty| ty.substitute_ty(self))
                            .collect::<Result<Vec<_>, _>>()?
                            .into(),
                        self.target_region.clone(),
                    )?;
                    inline_params.get_or_insert(new_target_region.params())
                }
                .next()
                .expect("Too few inline parameters");
                self.substitute_impl(
                    param.into_val(),
                    inline_param.into_val(),
                    false,
                    false,
                    false,
                    false,
                    false,
                )
                .expect("This should always succeed!");
            } else {
                return Err(Error::NoInlineError);
            }
        }
        if inline_params.is_none() {
            Ok(None)
        } else {
            Ok(Some(self.target_region.clone()))
        }
    }
    /// Register substitutes values for each value in a region, creating a new scope
    ///
    /// Return an error on a type/lifetime mismatch.
    /// If `inline` is true, create a new region with any leftover parameters, and return it if made
    /// If `inline` is false, return an error in this case.
    pub fn substitute_region<I>(
        &mut self,
        region: &Region,
        values: I,
        inline: bool,
    ) -> Result<Option<Region>, Error>
    where
        I: Iterator<Item = ValId>,
    {
        match self.domain_region.partial_cmp(region) {
            None => return Err(Error::IncomparableRegions),
            Some(Less) => {}
            Some(Equal) => return Err(Error::InvalidRedef),
            Some(Greater) => return Err(Error::DeepSub),
        }
        let old_self = if self.is_empty() {
            self.root_depth = region.depth();
            None
        } else {
            Some(self.clone())
        };
        self.domain_region = region.clone();
        let result = self.substitute_region_body(region, values, inline);
        if result.is_err() {
            if let Some(old_self) = old_self {
                *self = old_self
            } else {
                self.clear()
            }
        } else if let Some(old_self) = old_self {
            self.parents.push_back(old_self)
        }
        result
    }
    /// Try to quickly evaluate a given value in the current scope. Return None on failure
    #[inline]
    pub fn try_evaluate(&self, value: &ValId) -> Option<ValId> {
        // Check if the value's depth is too shallow to have been touched by this context
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
    /// Evaluate a value which is potentially within a deeper target region
    #[inline]
    pub fn evaluate_subvalue(&mut self, value: &ValId) -> Result<ValId, Error> {
        let value_region = value.region();
        match self.domain_region.partial_cmp(&value_region) {
            None => return Err(Error::IncomparableRegions),
            Some(Equal) | Some(Greater) => return self.evaluate(value),
            Some(Less) if value_region.depth() != self.depth() + 1 => return Err(Error::DeepSub),
            _ => {}
        };
        let new_target_region = Region::minimal_with(
            value_region
                .param_tys()
                .iter()
                .map(|ty| ty.substitute_ty(self))
                .collect::<Result<Vec<_>, Error>>()?
                .into(),
            self.target_region.region(),
        )?;
        self.substitute_region(
            value_region.as_region(),
            new_target_region.params().map(Value::into_val),
            false,
        )
        .expect("Substitution is valid!");
        let result = self.evaluate(value);
        self.pop();
        result
    }
}
