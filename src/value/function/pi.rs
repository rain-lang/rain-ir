/*!
Pi types
*/
use crate::value::{lifetime::Parametrized, typing::Typed, lifetime::{Live, LifetimeBorrow}, TypeId, TypeRef, UniverseId};

/// A pi type
#[derive(Debug)]
pub struct Pi {
    /// The result of this pi type
    result: Parametrized<TypeId>,
    /// The type of this pi type
    ty: UniverseId,
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