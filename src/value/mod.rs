/*!
`rain` values
*/
use crate::{debug_from_display, display_pretty};

pub mod primitive;
pub mod region;

/// An enumeration of possible `rain` values
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ValueEnum {}

debug_from_display!(ValueEnum);

display_pretty!(ValueEnum, |_, _| unimplemented!());

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Formatter};
    impl PrettyPrint for ValueEnum {
        fn prettyprint(
            &self,
            _printer: &mut PrettyPrinter,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }
}
