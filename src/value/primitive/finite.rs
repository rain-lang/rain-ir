/*!
Finite-valued types
*/

use crate::prettyprinter::tokens::*;
use crate::{debug_from_display, quick_pretty};
use num::ToPrimitive;
use ref_cast::RefCast;

/// A type with `n` values
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, RefCast)]
#[repr(transparent)]
pub struct Finite(pub u128);

debug_from_display!(Finite);
quick_pretty!(Finite, s, fmt => write!(fmt, "{}({})", KEYWORD_FINITE, s.0));

/// An index into a finite type
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Index {
    /// The type this index is part of
    ty: Finite,
    /// This index
    ix: u128,
}

impl Finite {
    /// Get an index into this type. Return an error if out of bounds
    pub fn ix<I: ToPrimitive>(self, ix: I) -> Result<Index, ()> {
        let ix = if let Some(ix) = ix.to_u128() {
            ix
        } else {
            return Err(());
        };
        Index::try_new(self, ix)
    }
}

debug_from_display!(Index);
quick_pretty!(Index, s, fmt => write!(fmt, "{}({})[{}]", KEYWORD_IX, s.ty, s.ix));

impl Index {
    /// Try to make a new index into a finite type. Return an error if out of bounds
    pub fn try_new(ty: Finite, ix: u128) -> Result<Index, ()> {
        if ix >= ty.0 {
            Err(())
        } else {
            Ok(Index { ty, ix })
        }
    }
    /// Get this index
    pub fn ix(&self) -> u128 {
        self.ix
    }
    /// Get the (finite) type of this index
    pub fn get_ty(&self) -> Finite {
        self.ty
    }
}
