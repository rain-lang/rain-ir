/*!
`rain` values
*/
use crate::{debug_from_display, display_pretty};
use std::sync::{Arc, Weak};

pub mod primitive;
pub mod region;

/// A reference-counted, hash-consed `rain` value
#[derive(Clone, Eq)]
pub struct ValId(Arc<ValueEnum>);

impl PartialEq for ValId {
    #[inline]
    fn eq(&self, other: &ValId) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

debug_from_display!(ValId);
display_pretty!(ValId, |_, _| unimplemented!());

/// A weak handle to a `rain` value
#[derive(Debug, Clone)]
pub struct WeakId(Weak<ValueEnum>);

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

    impl PrettyPrint for ValId {
        fn prettyprint(
            &self,
            _printer: &mut PrettyPrinter,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }
}
