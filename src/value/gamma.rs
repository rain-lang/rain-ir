/*!
Gamma nodes, representing pattern matching and primitive recursion
*/

use crate::value::{function::pi::Pi, lifetime::Region, typing::Typed, lifetime::{Live, Lifetime, LifetimeBorrow}, eval::Apply, TypeRef, ValId, VarId};
use crate::{debug_from_display, pretty_display};

/// A gamma node, representing pattern matching and primitive recursion
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Gamma {
    /// The branches of this gamma node
    branches: Box<[Branch]>,
    /// The dependencies of this gamma node, taken as a whole
    deps: Box<[ValId]>,
    /// The lifetime of this gamma node
    lifetime: Lifetime,
    /// The type of this gamma node
    //TODO: GammaPi?
    ty: VarId<Pi>,
}

impl Gamma {
    /// Get the branches of this gamma node
    pub fn branches(&self) -> &[Branch] {
        &self.branches
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

debug_from_display!(Gamma);
pretty_display!(Gamma, "{}{{ ... }}", prettyprinter::tokens::KEYWORD_GAMMA);

/// A builder for a gamma node
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GammaBuilder {
    /// The branches of this gamma node
    branches: Vec<Branch>,
    /// The desired type of this gamma node
    ty: VarId<Pi>,
    //TODO: completion check, etc...
}

impl GammaBuilder {
    /// Get the current branch-set of this gamma builder
    pub fn branches(&self) -> &[Branch] {
        &self.branches
    }
    /// Add a new branch to this gamma builder for a given pattern, which needs to be given a value
    /// Return an error on a mismatch between branch parameters and the desired gamma node type
    pub fn build_branch(&mut self, pattern: Pattern) -> Result<BranchBuilder, ()> {
        //TODO: type checking
        Ok(BranchBuilder {
            region: pattern.region(&self.branches),
            builder: self,
            pattern,
        })
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

/// A builder for a branch of a gamma node
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct BranchBuilder<'a> {
    /// The region corresponding to this branch's pattern
    region: Region,
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
    /// Get the pattern of this branch builder
    #[inline]
    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }
    /// Finish constructing branch with a given value, returning it's index in the builder on success.
    /// On failure, return an unchanged object to try again
    pub fn finish(self, value: ValId) -> Result<usize, BranchBuilder<'a>> {
        let ix = self.builder.branches.len();
        //TODO: region check...
        self.builder.branches.push(Branch {
            region: self.region,
            pattern: self.pattern,
            value, //TODO: check type, lifetimes, etc.
        });
        Ok(ix)
    }
}

/// A pattern for a gamma node branch
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Pattern {}

impl Pattern {
    /// Create the region for a given pattern given the current branch-set
    pub fn region(&self, _branches: &[Branch]) -> Region {
        match self {
            _ => unimplemented!(),
        }
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
