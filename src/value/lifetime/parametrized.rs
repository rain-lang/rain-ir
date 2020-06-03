/*!
A parametrized `rain` value of a given type
*/

use crate::value::lifetime::Region;
use crate::value::ValId;
use smallvec::SmallVec;

/// The size of a small list of parameter dependencies
pub const SMALL_PARAM_DEPS: usize = 2;

/// A parametrized value
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parametrized<V> {
    region: Region,
    value: V,
    deps: SmallVec<[ValId; SMALL_PARAM_DEPS]>,
}
