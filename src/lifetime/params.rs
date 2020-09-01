use super::*;

/// The size of a small vector of lifetime parameters
pub const SMALL_LIFETIME_PARAMS: usize = 2;

/// A vector of lifetime parameters
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LifetimeParams(pub SmallVec<[LifetimeId; SMALL_LIFETIME_PARAMS]>);

impl Deref for LifetimeParams {
    type Target = [LifetimeId];
    #[inline(always)]
    fn deref(&self) -> &[LifetimeId] {
        &self.0[..]
    }
}
