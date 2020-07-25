/*!
Gamma nodes, representing pattern matching and primitive recursion
*/

use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::function::{lambda::Lambda, pi::Pi};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::{Region, Regional};
use crate::typing::Typed;
use crate::value::{
    arr::ValSet, Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueEnum, VarId,
};
use crate::{
    debug_from_display, enum_convert, lifetime_region, pretty_display, substitute_to_valid,
};
use itertools::Itertools;
use thin_dst::ThinBox;

pub mod pattern;
use pattern::{Match, MatchedTy, Pattern};

/// A gamma node, representing pattern matching and primitive recursion
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Gamma {
    /// The branches of this gamma node
    branches: ThinBox<(), Branch>,
    /// The dependencies of this gamma node, taken as a whole.
    /// Sorted by address
    deps: ValSet,
    /// The lifetime of this gamma node
    lifetime: Lifetime,
    /// The type of this gamma node
    //TODO: GammaPi? Replace Lambda with Gamma?
    ty: VarId<Pi>,
}

impl Gamma {
    /// Get the branches of this gamma node
    pub fn branches(&self) -> &[Branch] {
        &self.branches.slice
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
    fn do_apply_in_ctx<'a>(
        &self,
        args: &'a [ValId],
        _inline: bool,
        ctx: Option<&mut EvalCtx>,
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
            if let Some(ctx) = ctx {
                return branch.do_apply_with_ctx(&inp.0, rest, true, ctx);
            } else {
                let eval_capacity = 0; //TODO
                let lt_capacity = 0; //TODO
                let mut ctx = EvalCtx::with_capacity(eval_capacity, lt_capacity);
                return branch.do_apply_with_ctx(&inp.0, rest, true, &mut ctx);
            }
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
        self.deps.len()
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        &self.deps[ix]
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
    /// The current pattern matched by this builder
    pattern: Pattern,
}

impl GammaBuilder {
    /// Create a new gamma node builder
    pub fn new(ty: VarId<Pi>) -> GammaBuilder {
        GammaBuilder {
            branches: Vec::new(),
            pattern: Pattern::empty(),
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
        self.pattern.take_disjunction(&branch.pattern);
        let ix = self.branches.len();
        self.branches.push(branch);
        Ok(Some(ix))
    }
    /// Push a branch onto this gamma node returning it's index *in the builder*, if any
    pub fn add(&mut self, branch: Branch) -> Result<Option<usize>, Error> {
        //TODO: check compatibility of this gamma node and branch
        if self.pattern.is_subset(&branch.pattern) {
            return Ok(None);
        }
        self.pattern.take_disjunction(&branch.pattern);
        let ix = self.branches.len();
        self.branches.push(branch);
        Ok(Some(ix))
    }
    /// Compute all the dependencies of this gamma builder, as of now, *without caching*. Slow!
    ///
    /// Dependencies are returned sorted by address, and deduplicated. The gamma lifetime is also returned.
    pub fn deps(&self) -> Result<(Lifetime, Vec<ValId>), (Error, Vec<ValId>)> {
        let mut lifetime = Lifetime::default();
        let mut has_error = None;
        let deps = self
            .branches
            .iter()
            .map(|branch| {
                let branch_lifetime =
                    lifetime.sep_conj(branch.deps().iter().map(|dep| dep.lifetime()));
                lifetime = match branch_lifetime.map(|branch_lifetime| &lifetime & branch_lifetime)
                {
                    Ok(Ok(lifetime)) => lifetime,
                    Err(err) | Ok(Err(err)) => {
                        has_error = Some(err);
                        Lifetime::default()
                    }
                };
                branch.deps().iter()
            })
            .kmerge_by(|a, b| a.as_ptr() < b.as_ptr())
            .dedup()
            .cloned()
            .collect();
        if let Some(err) = has_error {
            return Err((err, deps));
        }
        Ok((lifetime, deps))
    }
    /// Check whether this gamma node is complete
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.pattern.is_complete()
    }
    /// Finish constructing this gamma node
    /// On failure, return an unchanged object to try again, along with a reason
    pub fn finish(&self) -> Result<Gamma, Error> {
        // First, check completeness
        if !self.is_complete() {
            return Err(Error::IncompleteMatch);
        }

        // Then, actually make the gamma node
        let ty = self.ty.clone();
        let (lifetime, deps) = self.deps().map_err(|err| err.0)?;
        Ok(Gamma {
            deps: deps.into(),
            branches: ThinBox::new((), self.branches.iter().cloned()),
            lifetime,
            ty,
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
    /// The function corresponding to this branch
    func: ValId,
    /// The dependency set of this branch
    deps: ValSet,
}

impl Branch {
    /// Attempt to create a new branch from a pattern and a function
    pub fn try_new(pattern: Pattern, func: VarId<Lambda>) -> Result<Branch, Error> {
        //TODO: check if function is compatible with pattern
        let deps = func.depset().clone();
        let func = func.into_val();
        Ok(Branch {
            pattern,
            func,
            deps,
        })
    }
    /// Attempt to create a new branch with a constant (with respect to the pattern) value and a given input type vector
    pub fn try_const(pattern: Pattern, args: &[TypeId], value: ValId) -> Result<Branch, Error> {
        let MatchedTy(matched) = pattern.try_get_outputs(args)?;
        let region = Region::with(matched.into(), value.cloned_region());
        let func = Lambda::try_new(value, region)?;
        let deps = func.depset().clone();
        let func = func.into_val();
        Ok(Branch {
            pattern,
            func,
            deps,
        })
    }
    /// Return the dependencies of this branch
    pub fn deps(&self) -> &ValSet {
        &self.deps
    }
    /// Get the pattern of this branch
    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }
    /// Get the expression this branch evaluates
    pub fn expr(&self) -> &ValId {
        self.func.as_val()
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
