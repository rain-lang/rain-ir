/*!
Gamma nodes, representing pattern matching and primitive recursion
*/

use crate::value::{data::Constructor, function::pi::Pi, lifetime::Region, TypeId, ValId, VarId};
use crate::{debug_from_display, pretty_display};
use std::fmt::{self, Debug, Formatter};

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
    /// Match anything, and produce a parameter
    Any,
    /// Match anything without producing a parameter
    Null,
    /// Match a variant
    Variant(Variant),
    /// Recognize a pattern, binding it to a parameter
    Recognize(Recognize),
    /// Take the conjunction of a set of patterns, potentially binding the parameters of each.
    And(And),
    /// Take the disjunction of a set of patterns, potentially binding the parameters of each.
    Or(Or),
    /// Take the negation of a pattern
    Not(Not),
    /// Specify a multiset of parameters to use, ignoring the rest
    Select(Select),
    /// Specify the failure to match a given branch number *as a predicate*
    Failure(Failure),
    /// Specify the success of a given pattern *as a predicate*
    Success(Success),
    /// Bind a given pattern *as a reference*
    Ref(Ref),
    //TODO: range patterns, bit patterns...
}

/// A pattern which matches anything, and produces a parameter
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Any;

/// A pattern which matches anything, but produces nothing
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Null;

/// A pattern which recognizes its sub-pattern, and binds it to a parameter
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Recognize(pub Box<Pattern>);

/// A pattern which negates its sub-pattern *without* binding it to a parameter.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Not(pub Box<Pattern>);

/// A pattern corresponding to matching a variant
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Variant {
    variant: VarId<Constructor>,
    args: Box<[Pattern]>,
}

impl Variant {
    /// Attempt to create a new variant pattern
    pub fn try_new(variant: VarId<Constructor>, args: Box<[Pattern]>) -> Result<Variant, ()> {
        //TODO: check correspondence between arguments and variant...
        Ok(Variant {
            variant,
            args
        })
    }
}

impl Debug for Variant {
    fn fmt(&self, _fmt: &mut Formatter) -> Result<(), fmt::Error> {
        unimplemented!()
    }
}

/// A pattern corresponding to the conjunction of a set of patterns.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct And(pub Box<[Pattern]>);

/// A pattern corresponding to the disjunction of a set of patterns.
///
/// All patterns must have the *same* bound variables of the *same* type in the *same* order!
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Or(pub Box<[Pattern]>);

/// A pattern corresponding to using a multiset of the parameters of a pattern, in a given order
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Select {
    pattern: Box<Pattern>,
    parameters: Box<[usize]>,
}

/// A pattern corresponding to the failure of a set of branch numbers
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Failure(pub std::ops::Range<usize>);

/// A pattern corresponding to the success of a given pattern
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Success(pub Box<Pattern>);

/// A pattern corresponding to taking a reference of a given pattern
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Ref(pub Box<Pattern>);

/// Bind a given pattern as a reference
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Reference(pub Box<Pattern>);

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
