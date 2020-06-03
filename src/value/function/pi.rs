/*!
Pi types
*/
use crate::value::{lifetime::Parametrized, TypeId, UniverseId};

/// A pi type
#[derive(Debug)]
pub struct Pi {
    /// The result of this pi type
    result: Parametrized<TypeId>,
    /// The type of this pi type
    ty: UniverseId
}