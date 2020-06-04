/*!
Pi types
*/
use crate::value::{lifetime::Parametrized, typing::Typed, lifetime::{Live, LifetimeBorrow}, TypeId, TypeRef};

/// A pi type
#[derive(Debug)]
pub struct Pi {
    /// The result of this pi type
    result: Parametrized<TypeId>,
}

impl Typed for Pi {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.result.value().ty()
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