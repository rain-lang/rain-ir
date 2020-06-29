/*!
Gamma nodes, representing pattern matching and primitive recursion
*/

use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::function::{lambda::Lambda, pi::Pi};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::{Parameter, Region, RegionData};
use crate::typing::Typed;
use crate::value::{arr::ValSet, Error, NormalValue, TypeRef, ValId, Value, ValueEnum, VarId};
use crate::{debug_from_display, lifetime_region, pretty_display, substitute_to_valid};
use itertools::Itertools;
use thin_dst::ThinBox;

pub mod pattern;
use pattern::{Match, Pattern};

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
        let mut param_tys = self.ty.param_tys().iter();
        let mut ix = 0;
        while let Some(ty) = param_tys.next() {
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
pretty_display!(Gamma, "{}{{ ... }}", prettyprinter::tokens::KEYWORD_GAMMA);

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
    /// Add a new branch to this gamma builder for a given pattern, which needs to be given a value
    /// Return an error on a mismatch between branch parameters and the desired gamma node type
    pub fn build_branch(&mut self, pattern: Pattern) -> Result<BranchBuilder, Error> {
        let matched = pattern.try_match_ty(self.ty.borrow_var())?;
        let region = RegionData::with(
            matched.0.into(),
            self.ty
                .def_region()
                .parent()
                .cloned()
                .unwrap_or(Region::NULL),
        );
        let region = Region::new(region);
        let params = region.clone().params().collect();
        Ok(BranchBuilder {
            region,
            params,
            builder: self,
            pattern,
        })
    }
    /// Compute all the dependencies of this gamma builder, as of now, *without caching*. Slow!
    ///
    /// Dependencies are returned sorted by address, and deduplicated
    pub fn deps(&self) -> Vec<ValId> {
        self.branches
            .iter()
            .map(|branch| branch.deps().into_iter())
            .kmerge_by(|a, b| a.as_ptr() < b.as_ptr())
            .dedup()
            .cloned()
            .collect()
    }
    /// Check whether this gamma node is complete
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.pattern.is_complete()
    }
    /// Finish constructing this gamma node
    /// On failure, return an unchanged object to try again, along with a reason
    pub fn finish(mut self) -> Result<Gamma, (GammaBuilder, Error)> {
        // First, check completeness
        if !self.is_complete() {
            return Err((self, Error::IncompleteMatch));
        }

        // Then, actually make the gamma node
        let mut deps = self.deps();
        deps.shrink_to_fit();
        let lifetime = Lifetime::default()
            .intersect(deps.iter().map(|dep: &ValId| dep.lifetime()))
            .map_err(|_| Error::LifetimeError);
        let lifetime = match lifetime {
            Ok(lifetime) => lifetime,
            Err(err) => return Err((self, err)),
        };
        self.branches.shrink_to_fit();
        Ok(Gamma {
            deps: deps.into(),
            branches: ThinBox::new((), self.branches.into_iter()),
            lifetime,
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
    /// The function corresponding to this branch
    func: VarId<Lambda>,
}

impl Branch {
    /// Attempt to create a new branch from a pattern and a function
    pub fn new(pattern: Pattern, func: VarId<Lambda>) -> Result<Branch, Error> {
        //TODO: check if function is compatible with pattern
        Ok(Branch {
            pattern,
            func
        })
    }
    /// Return the dependencies of this branch
    pub fn deps(&self) -> &ValSet {
        self.func.depset()
    }
    /// Get the pattern of this branch
    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }
    /// Get the result of this branch
    pub fn result(&self) -> &ValId {
        &self.func.result()
    }
    /// Get the defining region of this branch
    pub fn def_region(&self) -> &Region {
        &self.func.def_region()
    }
    /// Get the function corresponding to this branch
    pub fn func(&self) -> &VarId<Lambda> {
        &self.func
    }
    /// Evaluate a branch with a given argument vector and context
    fn do_apply_with_ctx<'a>(
        &self,
        args: &[ValId],
        rest: &'a [ValId],
        inline: bool,
        ctx: &mut EvalCtx,
    ) -> Result<Application<'a>, Error> {
        // Substitute
        let region = ctx.push_region(
            self.def_region(),
            args.iter().cloned(),
            !ctx.is_checked(),
            inline,
        )?;

        // Evaluate the result
        let result = ctx.evaluate(self.result());
        // Pop the evaluation context
        ctx.pop();
        let result = result?;

        if let Some(region) = region {
            Lambda::try_new(result, region)
                .map(|lambda| Application::Success(rest, lambda.into()))
                .map_err(|_| Error::IncomparableRegions)
        } else {
            Ok(Application::Success(rest, result))
        }
    }
}

/// A builder for a branch of a gamma node
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct BranchBuilder<'a> {
    /// This branch's region
    region: Region,
    /// This branch's parameters
    params: Vec<Parameter>,
    /// The pattern corresponding to this branch
    pattern: Pattern,
    /// The builder for this branch
    builder: &'a mut GammaBuilder,
}

impl<'a> BranchBuilder<'a> {
    /// Get the region of this branch builder
    #[inline]
    pub fn region(&self) -> &Region {
        &self.region
    }
    /// Get the parameters of this branch builder
    #[inline]
    pub fn params(&self) -> &[Parameter] {
        &self.params
    }
    /// Get the pattern of this branch builder
    #[inline]
    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }
    /// Finish constructing branch with a given value, returning it's index in the builder on success.
    /// On failure, return an unchanged object to try again, along with a reason
    pub fn finish(self, value: ValId) -> Result<usize, (BranchBuilder<'a>, Error)> {
        let ix = self.builder.branches.len();
        let func = match Lambda::try_new(value, self.region.clone()) {
            Ok(func) => func,
            Err(err) => return Err((self, err)),
        };
        //TODO: check `value`'s type, region, lifetimes, etc.
        self.builder.pattern.take_disjunction(&self.pattern);
        self.builder.branches.push(Branch {
            pattern: self.pattern,
            func: func.into(),
        });
        Ok(ix)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::builder::Builder;
    use crate::region::Regional;
    use crate::value::expr::Sexpr;
    use std::convert::TryInto;

    #[test]
    fn not_as_gamma_works() {
        // Initialize the gamma node builder
        let mut builder = Builder::<&str>::new();
        let unary: VarId<Pi> = builder
            .parse_expr("#pi |_: #bool| #bool")
            .expect("The unary type is valid")
            .1
            .try_into()
            .expect("The unary type is a pi type");
        let mut gamma_builder = GammaBuilder::new(unary.clone());
        assert!(!gamma_builder.is_complete());
        assert_eq!(gamma_builder.branches().len(), 0);

        // Add the `#true` branch, mapping to `#false`
        let branch_builder = gamma_builder
            .build_branch(true.into())
            .expect("#true is a valid branch for #bool");
        assert_eq!(branch_builder.region(), unary.def_region());
        assert_eq!(branch_builder.params().len(), 1);
        assert_eq!(
            branch_builder
                .finish(false.into())
                .expect("#false is a valid result for #bool"),
            0
        );
        assert!(!gamma_builder.is_complete());
        assert_eq!(gamma_builder.branches().len(), 1);

        // Add the `#false` branch, mapping to `#true`
        let branch_builder = gamma_builder
            .build_branch(false.into())
            .expect("#false is a valid branch for #bool");
        assert_eq!(branch_builder.region(), unary.def_region());
        assert_eq!(branch_builder.params().len(), 1);
        assert_eq!(
            branch_builder
                .finish(true.into())
                .expect("#true is a valid result for #bool"),
            1
        );
        assert!(gamma_builder.is_complete());
        assert_eq!(gamma_builder.branches().len(), 2);

        // Complete gamma node construction
        let gamma = gamma_builder
            .finish()
            .expect("This is a complete gamma node");

        assert_eq!(gamma.branches().len(), 2);
        assert_eq!(gamma.region(), Region::NULL);

        let gamma = gamma.into_val();

        for t in [true, false].iter().copied() {
            assert_eq!(
                Sexpr::try_new(vec![gamma.clone(), t.into()])
                    .expect("Valid application")
                    .into_val(),
                (!t).into_val()
            )
        }
    }
}
