/*!
The `rain` type system
*/
use super::{TypeId, TypeRef, UniverseRef, Value};

/// A trait implemented by `rain` values with a type
pub trait Typed {
    /// Compute the type of this `rain` value
    fn ty(&self) -> TypeRef;
}

/// A trait implemented by `rain` values which are a type
pub trait Type: Into<TypeId> + Value {
    /// Get the universe of this type
    fn universe(&self) -> UniverseRef;
    /// Get whether this type is a universe
    fn is_universe(&self) -> bool;
}