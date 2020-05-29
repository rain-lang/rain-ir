/*!
`rain` values
*/
use crate::{debug_from_display, display_pretty};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use triomphe::Arc;

pub mod expr;
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

impl Hash for ValId {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

debug_from_display!(ValId);
display_pretty!(ValId, |_, _| unimplemented!());

/// A reference-counted, hash-consed `rain` type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TypeId(ValId);

debug_from_display!(TypeId);
display_pretty!(TypeId, |fmt, ty| write!(fmt, "{}", ty.0));

/// An enumeration of possible `rain` values
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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

    impl PrettyPrint for TypeId {
        fn prettyprint(
            &self,
            printer: &mut PrettyPrinter,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            self.0.prettyprint(printer, fmt)
        }
    }
}
