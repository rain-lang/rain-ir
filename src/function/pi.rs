/*!
Pi types
*/
use crate::eval::{Apply, EvalCtx, Substitute};
use crate::lifetime::{LifetimeBorrow, Live};
use crate::region::{Parameter, Parametrized, Region, RegionBorrow, Regional};
use crate::typing::{Type, Typed};
use crate::value::{
    arr::TyArr, Error, NormalValue, TypeId, TypeRef, UniverseId, UniverseRef, ValId, Value,
    ValueData, ValueEnum,
};
use crate::{debug_from_display, pretty_display, substitute_to_valid};

/// A pi type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Pi {
    /// The result of this pi type
    result: Parametrized<TypeId>,
    /// The type of this pi type
    ty: UniverseId,
}

impl Pi {
    /// Create a new pi type from a parametrized `TypeId`
    pub fn new(result: Parametrized<TypeId>) -> Pi {
        let ty = Self::universe(&result);
        Pi { ty, result }
    }
    /// Get the type associated with a parametrized `ValId`
    pub fn ty(param: &Parametrized<ValId>) -> Pi {
        Self::new(param.ty())
    }
    /// Get the universe associated with a parametrized `TypeId`
    pub fn universe(param: &Parametrized<TypeId>) -> UniverseId {
        param
            .value()
            .universe()
            .union_all(param.def_region().iter().map(|ty| ty.universe()))
    }
    /// Attempt to create a new pi type from a region and type
    pub fn try_new(value: TypeId, region: Region) -> Result<Pi, Error> {
        Ok(Self::new(Parametrized::try_new(value, region)?))
    }
    /// Get the result of this pi type
    #[inline]
    pub fn result(&self) -> &TypeId {
        self.result.value()
    }
    /// Get the defining region of this pi type
    #[inline]
    pub fn def_region(&self) -> &Region {
        self.result.def_region()
    }
    /// Get the parameter types of this pi type
    #[inline]
    pub fn param_tys(&self) -> &TyArr {
        self.def_region().param_tys()
    }
    /// Get the parameters of this pi type
    //TODO: parameter lifetimes...
    #[inline]
    pub fn params(&self) -> impl Iterator<Item = Parameter> + ExactSizeIterator {
        self.def_region().clone().params()
    }
}

impl Typed for Pi {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Live for Pi {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.result.lifetime()
    }
}

impl Regional for Pi {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.result.region()
    }
}

impl Apply for Pi {
    //TODO: allow limited pi-application?
}

impl Value for Pi {
    #[inline]
    fn no_deps(&self) -> usize {
        self.result.deps().len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self.result.deps()[ix]
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Pi(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for Pi {}

impl Type for Pi {
    #[inline]
    fn is_universe(&self) -> bool {
        false
    }
    #[inline]
    fn universe(&self) -> UniverseRef {
        self.ty.borrow_var()
    }
}

impl Substitute for Pi {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Pi, Error> {
        Ok(Pi::new(self.result.substitute(ctx)?))
    }
}

substitute_to_valid!(Pi);
debug_from_display!(Pi);
pretty_display!(Pi, "#pi|...| {...}");

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Pi {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "#pi")?;
            self.result.prettyprint(printer, fmt)
        }
    }
}
