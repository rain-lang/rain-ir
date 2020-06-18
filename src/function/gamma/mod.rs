/*!
Gamma nodes, representing pattern matching and primitive recursion
*/

use crate::eval::{Apply, EvalCtx, Substitute};
use crate::function::pi::Pi;
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::{Parameter, Region, RegionData, Regional};
use crate::typing::Typed;
use crate::value::{arr::ValSet, Error, NormalValue, TypeRef, ValId, Value, ValueEnum, VarId};
use crate::{debug_from_display, lifetime_region, pretty_display, substitute_to_valid};
use itertools::Itertools;
use std::ops::Deref;
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
    //TODO: again, pretty important, right?
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
    /// The region corresponding to this branch
    region: Region,
    /// The pattern of this branch
    pattern: Pattern,
    /// The value of this branch
    value: ValId,
}

impl Branch {
    /// Return the dependencies of this branch
    pub fn deps(&self) -> Vec<ValId> {
        use std::cmp::Ordering::*;
        match self
            .value
            .lifetime()
            .region()
            .partial_cmp(self.region.deref())
        {
            None | Some(Greater) => panic!(
                "Impossible: region mismatch should have been caught by BranchBuilder as error"
            ),
            Some(Equal) => self
                .value
                .deps()
                .collect_deps(..self.value.depth(), |_| true),
            Some(Less) => vec![self.value.clone()],
        }
    }
    /// Return the dependencies of this branch, sorted by address
    pub fn sorted_deps(&self) -> Vec<ValId> {
        let mut deps = self.deps();
        deps.sort_unstable_by_key(|v| v.as_ptr());
        deps
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
        //TODO: check `value`'s type, region, lifetimes, etc.
        self.builder.pattern.take_disjunction(&self.pattern);
        self.builder.branches.push(Branch {
            region: self.region,
            pattern: self.pattern,
            value,
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
    use std::convert::TryInto;
    #[test]
    fn not_as_gamma_works() {
        // Initialize the gamma builder
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

        //TODO: gamma node evaluation
    }
}
