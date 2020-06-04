/*!
Lambda functions
*/
use crate::{debug_from_display, pretty_display};
use super::pi::Pi;
use crate::value::{lifetime::Parametrized, ValId, VarId};

/// A lambda function
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Lambda {
    /// The result of this lambda function
    result: Parametrized<ValId>,
    /// The type of this lambda function
    ty: VarId<Pi>,
}

debug_from_display!(Lambda);
pretty_display!(Lambda, "#lambda |...| {...}");

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Lambda {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }
}
