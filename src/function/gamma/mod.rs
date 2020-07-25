/*!
Gamma nodes, representing pattern matching and primitive recursion
*/

use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::function::{lambda::Lambda, pi::Pi};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::{Region, Regional};
use crate::typing::Typed;
use crate::value::{Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueEnum, VarId};
use crate::{
    debug_from_display, enum_convert, lifetime_region, pretty_display, substitute_to_valid,
};

pub mod pattern;
use pattern::{Match, MatchedTy, Pattern};

/// A gamma node, representing pattern matching and primitive recursion
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Gamma {
    /// The branches of this gamma node
    branches: Box<[Branch]>,
    /// The lifetime of this gamma node
    lifetime: Lifetime,
    /// The type of this gamma node
    ty: VarId<Pi>,
}

impl Gamma {
    /// Get the branches of this gamma node
    pub fn branches(&self) -> &[Branch] {
        &self.branches
    }
    /// Get the type of this gamma node, guaranteed to be a pi type
    pub fn get_ty(&self) -> &VarId<Pi> {
        &self.ty
    }
}

impl Typed for Gamma {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
}

impl Live for Gamma {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

impl Apply for Gamma {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        let param_tys = self.ty.param_tys().iter();
        let mut ix = 0;
        for ty in param_tys {
            if ix >= args.len() {
                // Incomplete gamma application
                unimplemented!()
            }
            if args[ix].ty() != ty.borrow_ty() {
                return Err(Error::TypeMismatch);
            }
            ix += 1;
        }
        // Successful application
        let rest = &args[ix..];
        let inp = &args[..ix];
        for branch in self.branches() {
            let inp = if let Ok(inp) = branch.pattern().try_match(inp) {
                inp
            } else {
                continue;
            };
            let ctx = ctx.get_or_insert_with(|| {
                let eval_capacity = 0; //TODO
                let lt_capacity = 0; //TODO
                EvalCtx::with_capacity(eval_capacity, lt_capacity)
            });
            return branch.do_apply_with_ctx(&inp.0, rest, true, ctx);
        }
        panic!(
            "Complete gamma node has no matching branches!\nNODE: {:#?}\nARGV: {:#?}",
            self, args
        );
    }
}

impl Substitute for Gamma {
    fn substitute(&self, _ctx: &mut EvalCtx) -> Result<Gamma, Error> {
        unimplemented!()
    }
}

impl From<Gamma> for NormalValue {
    fn from(g: Gamma) -> NormalValue {
        NormalValue(ValueEnum::Gamma(g))
    }
}

impl Value for Gamma {
    fn no_deps(&self) -> usize {
        self.branches.len()
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        self.branches[ix].expr()
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Gamma(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

lifetime_region!(Gamma);

substitute_to_valid!(Gamma);

debug_from_display!(Gamma);
pretty_display!(Gamma, "{}{{ ... }}", crate::tokens::KEYWORD_GAMMA);
enum_convert! {
    impl InjectionRef<ValueEnum> for Gamma {}
    impl TryFrom<NormalValue> for Gamma { as ValueEnum, }
    impl TryFromRef<NormalValue> for Gamma { as ValueEnum, }
}

/// A builder for a gamma node
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GammaBuilder {
    /// The branches of this gamma node
    branches: Vec<Branch>,
    /// The desired type of this gamma node
    ty: VarId<Pi>,
    /// The lifetime of this gamma node
    lifetime: Lifetime,
    /// The current pattern matched by this builder
    pattern: Pattern,
}

impl GammaBuilder {
    /// Create a new gamma node builder
    pub fn new(ty: VarId<Pi>) -> GammaBuilder {
        GammaBuilder {
            branches: Vec::new(),
            pattern: Pattern::empty(),
            lifetime: Lifetime::STATIC,
            ty,
        }
    }
    /// Get the current branch-set of this gamma builder
    pub fn branches(&self) -> &[Branch] {
        &self.branches
    }
    /// Push a constant branch onto this gamma node for a given pattern
    pub fn add_const(&mut self, pattern: Pattern, value: ValId) -> Result<Option<usize>, Error> {
        if self.pattern.is_subset(&pattern) {
            return Ok(None);
        }
        let branch = Branch::try_const(pattern, self.ty.param_tys(), value)?;
        self.add_unchecked(branch).map(Some)
    }
    /// Push a branch onto this gamma node returning it's index *in the builder*, if any
    pub fn add(&mut self, branch: Branch) -> Result<Option<usize>, Error> {
        //TODO: check compatibility of this gamma node and branch
        if self.pattern.is_subset(&branch.pattern) {
            return Ok(None);
        }
        self.add_unchecked(branch).map(Some)
    }
    /// Push a branch onto this gamma node, without checking completeness or subset factors
    fn add_unchecked(&mut self, branch: Branch) -> Result<usize, Error> {
        let mut temp = Lifetime::STATIC;
        std::mem::swap(&mut self.lifetime, &mut temp);
        self.lifetime = (temp & branch.expr().lifetime())?;
        self.pattern.take_disjunction(&branch.pattern);
        let ix = self.branches.len();
        self.branches.push(branch);
        Ok(ix)
    }
    /// Check whether this gamma node is complete
    ///
    /// If this returns `true`, `finish` will always succeed
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.pattern.is_complete()
    }
    /// Finish constructing this gamma node
    pub fn finish(self) -> Result<Gamma, Error> {
        // First, check completeness
        if !self.is_complete() {
            return Err(Error::IncompleteMatch);
        }

        // Then, actually make the gamma node
        Ok(Gamma {
            branches: self.branches.into_boxed_slice(),
            lifetime: self.lifetime,
            ty: self.ty,
        })
    }
    /// Get the current pattern matched by this builder
    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }
}

/// A branch of a gamma node
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Branch {
    /// The pattern of this branch
    pattern: Pattern,
    /// The expression corresponding to this branch
    expr: ValId,
}

impl Branch {
    /// Attempt to create a new branch from a pattern and a function
    pub fn try_new(pattern: Pattern, func: VarId<Lambda>) -> Result<Branch, Error> {
        //TODO: check if function is compatible with pattern
        let expr = func.into_val();
        Ok(Branch { pattern, expr })
    }
    /// Attempt to create a new branch with a constant (with respect to the pattern) value and a given input type vector
    pub fn try_const(pattern: Pattern, args: &[TypeId], value: ValId) -> Result<Branch, Error> {
        let MatchedTy(matched) = pattern.try_get_outputs(args)?;
        if matched.len() == 0 {
            return Ok(Branch {
                pattern,
                expr: value,
            });
        }
        let region = Region::with(matched.into(), value.cloned_region());
        let func = Lambda::try_new(value, region)?;
        let expr = func.into_val();
        Ok(Branch { pattern, expr })
    }
    /// Get the pattern of this branch
    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }
    /// Get the expression this branch evaluates
    pub fn expr(&self) -> &ValId {
        self.expr.as_val()
    }
    /// Evaluate a branch with a given argument vector and context
    fn do_apply_with_ctx<'a>(
        &self,
        _args: &[ValId],
        _rest: &'a [ValId],
        _inline: bool,
        _ctx: &mut EvalCtx,
    ) -> Result<Application<'a>, Error> {
        unimplemented!()
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Gamma {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "UNIMPLEMENTED!")
        }
    }
}
