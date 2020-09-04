use super::*;

/// The size of a small vector of lifetime parameters
pub const SMALL_LIFETIME_PARAMS: usize = 2;

/// A vector of lifetime parameters
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct LifetimeParams(pub SmallVec<[Group; SMALL_LIFETIME_PARAMS]>);

impl Deref for LifetimeParams {
    type Target = [Group];
    #[inline(always)]
    fn deref(&self) -> &[Group] {
        &self.0[..]
    }
}
