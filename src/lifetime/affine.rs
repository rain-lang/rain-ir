/*!
Affine lifetimes
*/
use super::*;
use crate::region::Regional;
use crate::value::ValId;
use fxhash::FxBuildHasher;
use im::{hashmap::Entry, HashMap};

/// The data describing a purely affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AffineData {
    /// The affine data
    pub(super) data: HashMap<Color, Affine, FxBuildHasher>,
    /// Whether this data, taken together, is affine
    pub(super) affine: bool,
}

impl Default for AffineData {
    fn default() -> AffineData {
        AffineData {
            data: HashMap::default(),
            affine: false,
        }
    }
}

impl AffineData {
    /// Create an affine lifetime from a single obligation
    pub fn unit(color: Color, affinity: Affine) -> AffineData {
        let affine = affinity.is_affine();
        let mut data = HashMap::default();
        data.insert(color, affinity);
        AffineData { data, affine }
    }
    /// Create an affine lifetime only owning a given color
    pub fn owns(color: Color) -> AffineData {
        Self::unit(color, Affine::Owned)
    }
    /// Create an affine lifetime only borrowing a given color from a source
    pub fn borrows(color: Color, source: ValId) -> AffineData {
        Self::unit(color, Affine::Borrowed(source))
    }
    /// Take the separating conjunction of this lifetime with another
    ///
    /// Leaves this lifetime in an undetermined but valid state on failure
    pub fn sep_conj(&mut self, other: &AffineData) -> Result<(), Error> {
        if self.is_static() {
            *self = other.clone();
            return Ok(());
        }
        for (color, affinity) in other.data.iter() {
            match self.data.entry(color.clone()) {
                Entry::Occupied(mut o) => {
                    let other_affinity = o.get();
                    let new_affinity = affinity.sep_conj(other_affinity)?;
                    if new_affinity != *affinity {
                        o.insert(new_affinity);
                    }
                }
                Entry::Vacant(v) => {
                    v.insert(affinity.clone());
                }
            }
        }
        Ok(())
    }
    /// Take the disjunction of this lifetime with another
    ///
    /// Leaves this lifetime in an undetermined but valid state on failure
    pub fn disj(&mut self, other: &AffineData) -> Result<(), Error> {
        if self.is_static() {
            *self = other.clone();
            return Ok(());
        }
        for (color, affinity) in other.data.iter() {
            match self.data.entry(color.clone()) {
                Entry::Occupied(mut o) => {
                    let other_affinity = o.get();
                    let new_affinity = affinity.disj(other_affinity)?;
                    if new_affinity != *affinity {
                        o.insert(new_affinity);
                    }
                }
                Entry::Vacant(v) => {
                    v.insert(affinity.clone());
                }
            }
        }
        Ok(())
    }
    /// Whether this lifetime is the static lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.data.is_empty()
    }
    /// Whether data described by this lifetime is affine
    ///
    /// Non affine data is guaranteed to be equal to itself under self-intersection, while
    /// self-intersection of affine data is always an error
    #[inline]
    pub fn is_affine(&self) -> bool {
        self.affine
    }
    /// Whether this lifetime contains any mappings
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// The number of mappings this lifetime contains
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// Get the region of this lifetime
    #[inline]
    pub fn region(&self) -> Result<Option<RegionBorrow>, Error> {
        let mut keys = self.data.keys();
        let mut min = if let Some(first) = keys.next() {
            first.region()
        } else {
            return Ok(None);
        };
        for color in keys {
            let region = color.region();
            match region.partial_cmp(&min) {
                Some(Ordering::Less) => min = region,
                Some(_) => {}
                None => return Err(Error::IncomparableRegions),
            }
        }
        Ok(min)
    }
}

/// The data describing an affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Affine {
    /// Owns an affine type completely
    Owned,
    /// Borrows an affine type from a source
    Borrowed(ValId),
    //TODO: elemental ownership
}

impl Affine {
    /// Borrow this lifetime at a given source point
    pub fn borrow_from(&self, source: ValId) -> Result<Affine, Error> {
        match self {
            Affine::Owned => Ok(Affine::Borrowed(source)),
            Affine::Borrowed(b) => {
                if *b == source {
                    Ok(Affine::Borrowed(source))
                } else {
                    Err(Error::BorrowingMismatch)
                }
            }
        }
    }
    /// Take the separating conjunction of this lifetime with another
    pub fn sep_conj(&self, other: &Affine) -> Result<Affine, Error> {
        use Affine::*;
        match (self, other) {
            (Owned, Owned) => Err(Error::AffineUsed),
            (Owned, Borrowed(_)) | (Borrowed(_), Owned) => Err(Error::BorrowUsed),
            (Borrowed(l), Borrowed(r)) => {
                if l == r {
                    Ok(Borrowed(l.clone()))
                } else {
                    Err(Error::BorrowedMismatch)
                }
            }
        }
    }
    /// Take the disjunction of this lifetime with another
    pub fn disj(&self, other: &Affine) -> Result<Affine, Error> {
        use Affine::*;
        match (self, other) {
            (Owned, _) => Ok(Owned),
            (_, Owned) => Ok(Owned),
            (Borrowed(l), Borrowed(r)) => {
                if l == r {
                    Ok(Borrowed(l.clone()))
                } else {
                    Err(Error::BorrowedMismatch)
                }
            }
        }
    }
    /// Whether this lifetime is affine in itself
    pub fn is_affine(&self) -> bool {
        use Affine::*;
        match self {
            Owned => true,
            Borrowed(_) => false,
        }
    }
    /// Take the separating conjunction of this affine lifetime with itself
    pub fn star_self(&self) -> Result<(), Error> {
        use Affine::*;
        match self {
            Owned => Err(Error::LifetimeError),
            Borrowed(_) => Ok(()),
        }
    }
}

impl Mul for AffineData {
    type Output = Result<AffineData, Error>;
    #[inline]
    fn mul(self, other: AffineData) -> Result<AffineData, Error> {
        //TODO: think about this...
        self * &other
    }
}

impl Mul<&'_ AffineData> for AffineData {
    type Output = Result<AffineData, Error>;
    #[inline]
    fn mul(self, other: &AffineData) -> Result<AffineData, Error> {
        other * self
    }
}

impl Mul for &'_ AffineData {
    type Output = Result<AffineData, Error>;
    #[inline]
    fn mul(self, other: &AffineData) -> Result<AffineData, Error> {
        self * other.clone()
    }
}

impl Mul<AffineData> for &'_ AffineData {
    type Output = Result<AffineData, Error>;
    #[inline]
    fn mul(self, mut other: AffineData) -> Result<AffineData, Error> {
        other.sep_conj(self)?;
        Ok(other)
    }
}

impl Add for AffineData {
    type Output = Result<AffineData, Error>;
    #[inline]
    fn add(self, other: AffineData) -> Result<AffineData, Error> {
        //TODO: think about this
        self + &other
    }
}

impl Add<&'_ AffineData> for AffineData {
    type Output = Result<AffineData, Error>;
    #[inline]
    fn add(mut self, other: &AffineData) -> Result<AffineData, Error> {
        self.disj(other)?;
        Ok(self)
    }
}

impl Add for &'_ AffineData {
    type Output = Result<AffineData, Error>;
    #[inline]
    fn add(self, other: &AffineData) -> Result<AffineData, Error> {
        self.clone() + other
    }
}

impl Add<AffineData> for &'_ AffineData {
    type Output = Result<AffineData, Error>;
    #[inline]
    fn add(self, other: AffineData) -> Result<AffineData, Error> {
        other + self
    }
}

impl Mul for Affine {
    type Output = Result<Affine, Error>;
    fn mul(self, other: Affine) -> Result<Affine, Error> {
        self.sep_conj(&other)
    }
}

impl Mul<&'_ Affine> for Affine {
    type Output = Result<Affine, Error>;
    fn mul(self, other: &Affine) -> Result<Affine, Error> {
        self.sep_conj(other)
    }
}

impl Mul for &'_ Affine {
    type Output = Result<Affine, Error>;
    fn mul(self, other: &Affine) -> Result<Affine, Error> {
        self.sep_conj(other)
    }
}

impl Mul<Affine> for &'_ Affine {
    type Output = Result<Affine, Error>;
    fn mul(self, other: Affine) -> Result<Affine, Error> {
        self.sep_conj(&other)
    }
}

impl Add for Affine {
    type Output = Result<Affine, Error>;
    fn add(self, other: Affine) -> Result<Affine, Error> {
        self.disj(&other)
    }
}

impl Add<&'_ Affine> for Affine {
    type Output = Result<Affine, Error>;
    fn add(self, other: &Affine) -> Result<Affine, Error> {
        self.disj(other)
    }
}

impl Add for &'_ Affine {
    type Output = Result<Affine, Error>;
    fn add(self, other: &Affine) -> Result<Affine, Error> {
        self.disj(other)
    }
}

impl Add<Affine> for &'_ Affine {
    type Output = Result<Affine, Error>;
    fn add(self, other: Affine) -> Result<Affine, Error> {
        self.disj(&other)
    }
}
