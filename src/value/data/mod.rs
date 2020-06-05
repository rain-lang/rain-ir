/*!
`rain` data declarations and compound/inductive types
*/
use super::{NormalValue, PrivateValue};
use crate::{debug_from_display, pretty_display};
use ref_cast::RefCast;
use std::ops::Deref;

#[derive(Clone, Eq, PartialEq, Hash)]
/// A value which can be interpreted as a type constructor
pub struct Constructor(PrivateValue);

debug_from_display!(Constructor);
pretty_display!(Constructor, s, fmt => write!(fmt, "{}", s.deref()));

impl Deref for Constructor {
    type Target = NormalValue;
    fn deref(&self) -> &NormalValue {
        RefCast::ref_cast(&self.0)
    }
}

/// A record type with named members, supporting row typing
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Struct {}

/// An enumeration type, or algebraic sum, with named members
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Enum {}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Constructor {
        #[inline]
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            if let Some(name) = printer.lookup(self) {
                write!(fmt, "{}", name)
            } else {
                self.deref().prettyprint(printer, fmt)
            }
        }
    }
}
