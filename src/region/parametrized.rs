/*!
A parametrized `rain` value of a given type
*/

use crate::eval::{EvalCtx, Substitute};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::{Region, RegionBorrow, Regional};
use crate::typing::Typed;
use crate::value::{arr::ValSet, Error, TypeId, ValId, Value};
use smallvec::{smallvec, SmallVec};
use std::cmp::Ordering;
use std::convert::TryInto;

/// A parametrized value
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parametrized<V> {
    region: Region,
    value: V,
    deps: ValSet,
    lifetime: Lifetime,
}

impl<V: Value + Clone> Parametrized<V> {
    /**
    Attempt to create a new parametrized value. Return an error if the value does not lie in the desired region.
    */
    pub fn try_new(value: V, region: Region) -> Result<Parametrized<V>, Error> {
        use Ordering::*;
        match value.region().partial_cmp(&region) {
            None | Some(Greater) => Err(Error::IncomparableRegions),
            Some(Equal) => {
                let mut deps = value.deps().collect_deps(..value.depth(), |_| true);
                deps.shrink_to_fit();
                let lifetime = Lifetime::default()
                    .intersect_all(deps.iter().map(|dep: &ValId| dep.lifetime()))
                    .map_err(|_| Error::LifetimeError)?;
                Ok(Parametrized {
                    region,
                    value,
                    deps: deps.into_iter().collect(),
                    lifetime,
                })
            }
            Some(Less) => {
                let mut deps: SmallVec<[ValId; 0]> = smallvec![value.clone().into_val()];
                deps.shrink_to_fit();
                let lifetime = Lifetime::default()
                    .intersect_all(deps.iter().map(|dep: &ValId| dep.lifetime()))
                    .map_err(|_| Error::LifetimeError)?;
                Ok(Parametrized {
                    region,
                    value,
                    deps: deps.into_iter().collect(),
                    lifetime,
                })
            }
        }
    }
}

impl<V: Typed> Parametrized<V> {
    /**
    Get the parametrized type of this parametrized value
    */
    #[inline]
    pub fn ty(&self) -> Parametrized<TypeId> {
        let ty = self.value.ty().clone_ty();
        Parametrized::try_new(ty, self.region.clone())
            //TODO: think about this...
            .expect("A type should never be in a region a value is not!")
    }
}

impl<V> Parametrized<V> {
    /**
    Get the value being parametrized
    */
    #[inline]
    pub fn value(&self) -> &V {
        &self.value
    }
    /**
    Get the dependencies of this value
    */
    #[inline]
    pub fn deps(&self) -> &[ValId] {
        self.deps.as_slice()
    }
    /**
    Get the region in which this parametrized value is defined
    */
    #[inline]
    pub fn def_region(&self) -> &Region {
        &self.region
    }
    /**
    Decompose this `Parametrized` into its components
    */
    #[inline]
    pub fn destruct(self) -> (Region, V, ValSet, Lifetime) {
        (self.region, self.value, self.deps, self.lifetime)
    }
}

impl<V: Value> Parametrized<V> {
    /**
    Convert a parametrized value into another
    */
    pub fn into_value<U>(self) -> Parametrized<U>
    where
        U: Value,
        V: Into<U>,
    {
        Parametrized {
            region: self.region,
            value: self.value.into(),
            deps: self.deps,
            lifetime: self.lifetime,
        }
    }
    /**
    Try to convert a parametrized value into another
    */
    pub fn try_into_value<U>(self) -> Result<Parametrized<U>, V::Error>
    where
        U: Value,
        V: TryInto<U>,
    {
        Ok(Parametrized {
            region: self.region,
            value: self.value.try_into()?,
            deps: self.deps,
            lifetime: self.lifetime,
        })
    }
}

impl<V: Value> Live for Parametrized<V> {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

impl<V: Value> Regional for Parametrized<V> {
    fn region(&self) -> RegionBorrow {
        self.lifetime().get_region()
    }
}

impl<U, V> Substitute<Parametrized<U>> for Parametrized<V>
where
    V: Substitute<U> + Value,
    U: Value + Clone,
{
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Parametrized<U>, Error> {
        let value: U = self.value().substitute(ctx)?;
        Parametrized::try_new(value, self.def_region().clone())
    }
}

/// Prettyprinting implementation for parametrized values
#[cfg(feature = "prettyprinter")]
pub mod prettyprint {
    use super::*;
    use crate::prettyprinter::{tokens::*, PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    /// Prettyprint a value parametrized by a given region
    pub fn prettyprint_parametrized<I, V>(printer: &mut PrettyPrinter<I>, fmt: &mut Formatter, value: &V, region: &Region) -> Result<(), fmt::Error>
    where
        I: From<usize> + Display,
        V: PrettyPrint + Value
    {
        //TODO: print _ for parameters if all unused?
        write!(fmt, "{}", PARAM_OPEN)?;
        let mut first = true;
        for param in region.borrow_params() {
            if !first {
                write!(fmt, " ")?;
            }
            first = false;
            printer.prettyprint_index(fmt, ValId::<()>::from(param).borrow_val())?;
        }
        write!(fmt, "{} ", PARAM_CLOSE)?;
        printer.scoped_print(fmt, value)?;
        Ok(())
    }

    impl<V> PrettyPrint for Parametrized<V>
    where
        V: PrettyPrint + Value,
    {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            prettyprint_parametrized(printer, fmt, &self.value, &self.region)
        }
    }
}
