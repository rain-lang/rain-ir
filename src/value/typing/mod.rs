/*!
The `rain` type system
*/
use super::{TypeRef, UniverseRef};

/// A trait implemented by `rain` values with a type
pub trait Typed {
    /// Compute the type of this `rain` value
    fn ty(&self) -> TypeRef;
}

/// A trait implemented by `rain` values which are a type
pub trait Type {
    /// Get the universe of this type
    fn universe(&self) -> UniverseRef;
}