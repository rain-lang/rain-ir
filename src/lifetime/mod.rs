/*!
The `rain` lifetime system
*/

use crate::region::{data::RegionData};
use crate::value::Error;
use crate::value::{ValId, ValRef};
use elysees::Arc;
use fxhash::FxBuildHasher;
use indexmap::{map::Entry, IndexMap};
use itertools::{EitherOrBoth, Itertools};
use ptr_union::Union2;
use smallvec::SmallVec;
use std::iter::Copied;
use std::ops::Deref;

mod group;
pub use group::*;
mod params;
pub use params::*;
mod ctx;
pub use ctx::*;
mod data;
pub use data::*;

/// A `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Lifetime(Option<Union2<Arc<RegionData>, Arc<LifetimeData>>>);

impl Lifetime {
    /// The static `rain` lifetime
    pub const STATIC: Lifetime = Lifetime(None);
}