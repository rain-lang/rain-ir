/*!
A parametrized `rain` value of a given type
*/

use crate::value::lifetime::{Lifetime, LifetimeBorrow, Live, Region};
use crate::value::{typing::Typed, TypeId, ValId, Value};
use smallvec::{smallvec, SmallVec};
use std::cmp::Ordering;
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
    pub fn try_new(value: V, region: Region) -> Result<Parametrized<V>, ()> {
        use Ordering::*;
        match value.region().partial_cmp(region.deref()) {
            None | Some(Less) => Err(()),
            Some(Equal) => {
                let deps = value.deps().collect_deps(value.lifetime().depth());
                let lifetime =
                    Lifetime::default().intersect(deps.iter().map(|dep: &ValId| dep.lifetime()))?;
                Ok(Parametrized {
                    region,
                    value,
                    deps,
                    lifetime,
                })
            }
            Some(Greater) => {
                let deps = smallvec![value.clone().into()];
                let lifetime =
                    Lifetime::default().intersect(deps.iter().map(|dep: &ValId| dep.lifetime()))?;
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

impl<V: Value> Live for Parametrized<V> {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}
