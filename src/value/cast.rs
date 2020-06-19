/*!
`rain` value and lifetime casting
*/
use super::{Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueEnum};
use crate::eval::{Apply, EvalCtx, Substitute};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::{RegionBorrow, Regional};
use crate::typing::{Type, Typed};
use crate::{debug_from_display, pretty_display, substitute_to_valid};

/// A `rain` cast
#[derive(Clone, PartialEq, Eq, Hash)]
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
    #[inline]
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
    #[inline]
    pub fn new(value: ValId, ty: Option<TypeId>, lt: Option<Lifetime>) -> Result<Cast, Error> {
        Cast { value, ty, lt }.validate()
    }
    /// Create a lifetime cast
    #[inline]
    pub fn ltcast(value: ValId, lt: Lifetime) -> Result<Cast, Error> {
        Self::new(value, None, Some(lt))
    }
    /// Create a typecast
    #[inline]
    pub fn tycast(value: ValId, ty: TypeId) -> Result<Cast, Error> {
        Self::new(value, Some(ty), None)
    }
    /// Get the type being cast to
    #[inline]
    pub fn get_tycast(&self) -> Option<&TypeId> {
        self.ty.as_ref()
    }
    /// Get the lifetime being cast to
    #[inline]
    pub fn get_ltcast(&self) -> Option<&Lifetime> {
        self.lt.as_ref()
    }
    /// Get the underlying value being cast
    #[inline]
    pub fn value(&self) -> &ValId {
        &self.value
    }
    /// Take the underlying value of this cast
    #[inline]
    pub fn take_value(self) -> ValId {
        self.value
    }
}

impl Typed for Cast {
    fn is_ty(&self) -> bool {
        if let Some(ty) = &self.ty {
            ty.is_universe()
        } else {
            self.value.is_ty()
        }
    }
    fn ty(&self) -> TypeRef {
        if let Some(ty) = &self.ty {
            ty.borrow_ty()
        } else {
            self.value.ty()
        }
    }
}

impl Live for Cast {
    fn lifetime(&self) -> LifetimeBorrow {
        if let Some(lt) = &self.lt {
            lt.borrow_lifetime()
        } else {
            self.value.lifetime()
        }
    }
}

impl Regional for Cast {
    fn region(&self) -> RegionBorrow {
        if let Some(lt) = &self.lt {
            lt.region()
        } else {
            self.value.region()
        }
    }
}

impl Apply for Cast {
    //TODO: delegate here...
}

impl Substitute for Cast {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Cast, Error> {
        Ok(Cast {
            value: self.value.substitute(ctx)?,
            ty: self.ty.as_ref().map(|ty| ty.substitute(ctx)).transpose()?,
            //TODO: fix this bit...
            lt: self.lt.clone(),
        })
    }
}
//TODO: optimization may be possible here...
substitute_to_valid!(Cast);

impl Value for Cast {
    fn no_deps(&self) -> usize {
        1 + if self.ty.is_some() { 1 } else { 0 }
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        match ix {
            0 => &self.value,
            1 => self.ty.as_ref().expect("Expected typecast").as_val(),
            _ => panic!("Invalid dependency index {} into Cast", ix),
        }
    }
    fn into_enum(self) -> ValueEnum {
        self.into()
    }
    fn into_norm(self) -> NormalValue {
        self.into()
    }
    fn into_val(self) -> ValId {
        self.into()
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
