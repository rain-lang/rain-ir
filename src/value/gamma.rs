/*!
Gamma nodes, representing pattern matching and primitive recursion
*/

use crate::{debug_from_display, pretty_display};
use crate::value::{ValId, VarId, lifetime::Region, function::pi::Pi};

/// A gamma node, representing pattern matching and primitive recursion
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Gamma {
    /// The branches of this gamma node
    branches: Vec<Branch>,
    /// The dependencies of this gamma node, taken as a whole
    deps: Vec<ValId>,
    /// The type of this gamma node
    ty: VarId<Pi>
}

debug_from_display!(Gamma);
pretty_display!(Gamma, "{}{{ ... }}", prettyprinter::tokens::KEYWORD_GAMMA);

/// A branch of a gamma node
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Branch {
    /// The region corresponding to this branch
    region: Region,
    /// The pattern of this branch
    pattern: Pattern
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
    Recognize(Box<Pattern>),
    /// Take the conjunction of a set of patterns, potentially binding the parameters of each.
    And(And),
    /// Take the disjunction of a set of patterns, potentially binding the parameters of each.
    Or(Or),
    /// Take the negation of a pattern
    Not(Not),
    /// Specify a multiset of parameters to use, ignoring the rest
    Select(Select),
    /// Specify the failure to match a given branch number *as a predicate*
    Failure(usize),
    /// Specify the success of a given pattern *as a predicate*
    Success(Box<Pattern>),
    /// Bind a given pattern *as a reference*
    Ref(Box<Pattern>),
}

/// A pattern which recognizes its sub-pattern, and binds it to a parameter
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Recognize(pub Box<Pattern>);

/// A pattern which negates its sub-pattern *without* binding it to a parameter.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Not(pub Box<Pattern>);

/// A pattern corresponding to matching a variant
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Variant {
    //TODO: this
}

/// A pattern corresponding to the conjunction of a set of patterns.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct And(pub Vec<Pattern>);

/// A pattern corresponding to the disjunction of a set of patterns.
/// 
/// All patterns must have the *same* bound variables of the *same* type in the *same* order!
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Or(Vec<Pattern>);

/// A pattern corresponding to using a multiset of the parameters of a pattern, in a given order
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Select {
    //TODO: this
}

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
