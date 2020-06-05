/*!
A parametrized `rain` value of a given type
*/

use crate::value::eval::{self, EvalCtx, Substitute};
use crate::value::lifetime::{Lifetime, LifetimeBorrow, Live, Region};
use crate::value::{typing::Typed, TypeId, ValId, Value};
use smallvec::{smallvec, SmallVec};
use std::cmp::Ordering;
use std::convert::TryInto;
use std::ops::Deref;

/// The size of a small list of parameter dependencies
pub const SMALL_PARAM_DEPS: usize = 2;

/// A parametrized value
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parametrized<V> {
    region: Region,
    value: V,
    deps: SmallVec<[ValId; SMALL_PARAM_DEPS]>,
    lifetime: Lifetime,
}

impl<V: Value + Clone + Into<ValId>> Parametrized<V> {
    /**
    Attempt to create a new parametrized value. Return an error if the value does not lie in the desired region.
    */
    pub fn try_new(value: V, region: Region) -> Result<Parametrized<V>, eval::Error> {
        use Ordering::*;
        match value.region().partial_cmp(region.deref()) {
            None | Some(Greater) => Err(eval::Error::IncomparableRegions),
            Some(Equal) => {
                let deps = value.deps().collect_deps(value.lifetime().depth());
                let lifetime = Lifetime::default()
                    .intersect(deps.iter().map(|dep: &ValId| dep.lifetime()))
                    .map_err(|_| eval::Error::LifetimeError)?;
                Ok(Parametrized {
                    region,
                    value,
                    deps,
                    lifetime,
                })
            }
            Some(Less) => {
                let deps = smallvec![value.clone().into()];
                let lifetime = Lifetime::default()
                    .intersect(deps.iter().map(|dep: &ValId| dep.lifetime()))
                    .map_err(|_| eval::Error::LifetimeError)?;
                Ok(Parametrized {
                    region,
                    value,
                    deps,
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
    pub fn value(&self) -> &V {
        &self.value
    }
    /**
    Get the dependencies of this value
    */
    pub fn deps(&self) -> &[ValId] {
        &self.deps
    }
    /**
    Get the region in which this parametrized value is defined
    */
    pub fn def_region(&self) -> &Region {
        &self.region
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

impl<U, V> Substitute<Parametrized<U>> for Parametrized<V>
where
    V: Substitute<U> + Value,
    U: Value + Clone,
{
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Parametrized<U>, eval::Error> {
        let value: U = self.value().substitute(ctx)?;
        Parametrized::try_new(value, self.def_region().clone())
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{tokens::*, PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl<V> PrettyPrint for Parametrized<V>
    where
        V: PrettyPrint + Value,
    {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "{}", PARAM_OPEN)?;
            let mut first = true;
            for param in self.region.borrow_params() {
                if !first {
                    write!(fmt, " ")?;
                }
                first = false;
                printer.prettyprint_index(fmt, ValId::from(param).borrow_val())?;
            }
            write!(fmt, "{} ", PARAM_CLOSE)?;
            printer.scoped_print(fmt, &self.value)?;
            Ok(())
        }
    }
}
