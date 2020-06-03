/*!
A parametrized `rain` value of a given type
*/

use crate::value::lifetime::Region;
use crate::value::{ValId, Value};
use smallvec::{SmallVec, smallvec};
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
                unimplemented!()
            },
            Some(Greater) => {
                let deps = smallvec![value.clone().into()];
                Ok(Parametrized {
                    region,
                    value,
                    deps
                })
            }
        }
    }
}
