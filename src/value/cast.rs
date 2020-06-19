/*!
`rain` value and lifetime casting
*/
use super::{Error, TypeId, ValId};
use crate::lifetime::{Lifetime, Live};
use crate::typing::Typed;
use crate::{debug_from_display, pretty_display};

/// A `rain` cast
#[derive(PartialEq, Eq, Hash)]
pub struct Cast {
    /// The value being cast
    value: ValId,
    /// The type it is being cast to
    ty: Option<TypeId>,
    /// The lifetime it is being cast to
    lt: Option<Lifetime>,
}

impl Cast {
    /// Validate a cast
    fn validate(self) -> Result<Cast, Error> {
        if let Some(ty) = &self.ty {
            //TODO: subtyping check...
            if *ty != self.value.ty() {
                return Err(Error::TypeMismatch);
            }
        }
        if let Some(lt) = &self.lt {
            use std::cmp::Ordering::*;
            match lt.partial_cmp(&self.value.lifetime()) {
                None | Some(Less) => return Err(Error::LifetimeError),
                _ => {}
            }
        }
        Ok(self)
    }
    /// Create a new cast
    pub fn new(value: ValId, ty: Option<TypeId>, lt: Option<Lifetime>) -> Result<Cast, Error> {
        Cast { value, ty, lt }.validate()
    }
    /// Create a lifetime cast
    pub fn ltcast(value: ValId, lt: Lifetime) -> Result<Cast, Error> {
        Self::new(value, None, Some(lt))
    }
    /// Create a typecast
    pub fn tycast(value: ValId, ty: TypeId) -> Result<Cast, Error> {
        Self::new(value, Some(ty), None)
    }
}

debug_from_display!(Cast);
pretty_display!(Cast, "#cast(...)");

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Cast {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            //TODO: this
            write!(fmt, "#cast(...)")
        }
    }
}
