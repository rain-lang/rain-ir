/*!
Gamma nodes, representing pattern matching and primitive recursion
*/

use crate::value::{function::pi::Pi, lifetime::Region, TypeId, ValId, VarId};
use crate::{debug_from_display, pretty_display};

/// A gamma node, representing pattern matching and primitive recursion
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Gamma {
    /// The branches of this gamma node
    branches: Box<[Branch]>,
    /// The dependencies of this gamma node, taken as a whole
    deps: Box<[ValId]>,
    /// The type of this gamma node
    ty: VarId<Pi>,
}

impl Gamma {
    /// Try to create a new gamma node from a set of branches and a source type
    pub fn try_new(_branches: CompleteBranches) -> Result<Gamma, ()> {
        unimplemented!()
    }
}

debug_from_display!(Gamma);
pretty_display!(Gamma, "{}{{ ... }}", prettyprinter::tokens::KEYWORD_GAMMA);

/// A complete set of branches over a source type
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CompleteBranches {
    branches: Vec<Branch>,
    source_ty: TypeId,
}

/// A branch of a gamma node
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Branch {
    /// The region corresponding to this branch
    region: Region,
    /// The pattern of this branch
    pattern: Pattern,
}

impl Branch {
    /// Create a new branch from a given pattern, generating the region for it
    pub fn new(_pattern: Pattern) -> Pattern {
        unimplemented!()
    }
}

/// A pattern for a gamma node branch
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Pattern {
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
