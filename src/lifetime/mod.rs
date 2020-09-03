/*!
The `rain` lifetime system
*/

use crate::value::Error;
use crate::value::{ValId, ValRef};
use fxhash::FxBuildHasher;
use indexmap::{map::Entry, IndexMap};
use itertools::{EitherOrBoth, Itertools};
use smallvec::SmallVec;
use std::iter::Copied;
use std::ops::Deref;

mod group;
pub use group::*;
mod params;
pub use params::*;
mod ctx;
pub use ctx::*;