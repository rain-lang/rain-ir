/*!
A prettyprinter for `rain` programs
*/
use std::fmt::{self, Formatter};
use std::default::Default;

/// A prettyprinter for `rain` values
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PrettyPrinter {}

impl PrettyPrinter {
    /// Create a new prettyprinter
    pub fn new() -> PrettyPrinter {
        PrettyPrinter {}
    }
}

impl Default for PrettyPrinter {
    fn default() -> PrettyPrinter {
        Self::new()
    }
}

/// A value which can be prettyprinted
pub trait PrettyPrint {
    /// Prettyprint a value using a given printer
    fn prettyprint(
        &self,
        printer: &mut PrettyPrinter,
        fmt: &mut Formatter,
    ) -> Result<(), fmt::Error>;
}

/// Implement `PrettyPrint` using `Display`
#[macro_export]
macro_rules! prettyprint_by_display {
    ($t:ty) => {
        impl $crate::prettyprinter::PrettyPrint for $t {
            fn prettyprint(
                &self,
                printer: &mut $crate::prettyprinter::PrettyPrinter,
                fmt: &mut std::fmt::Formatter,
            ) -> Result<(), std::fmt::Error> {
            }
        }
    };
}
