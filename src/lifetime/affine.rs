/*!
Affine lifetimes
*/
use super::*;
use crate::value::ValId;

/// The data describing an affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Affine {
    /// Owns an affine type completely
    Owned,
    /// Borrows an affine type from a source
    Borrowed(ValId),
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
    pub fn star(&self, other: &Affine) -> Result<Affine, Error> {
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
    /// Take the conjunction of this lifetime with another
    pub fn conj(&self, other: &Affine) -> Result<Affine, Error> {
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
    /// Whether this lifetime is idempotent under separating conjunction
    pub fn idempotent(&self) -> bool {
        use Affine::*;
        match self {
            Owned => false,
            Borrowed(_) => true,
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

impl Mul for Affine {
    type Output = Result<Affine, Error>;
    fn mul(self, other: Affine) -> Result<Affine, Error> {
        self.star(&other)
    }
}

impl Mul<&'_ Affine> for Affine {
    type Output = Result<Affine, Error>;
    fn mul(self, other: &Affine) -> Result<Affine, Error> {
        self.star(other)
    }
}

impl Mul for &'_ Affine {
    type Output = Result<Affine, Error>;
    fn mul(self, other: &Affine) -> Result<Affine, Error> {
        self.star(other)
    }
}

impl Mul<Affine> for &'_ Affine {
    type Output = Result<Affine, Error>;
    fn mul(self, other: Affine) -> Result<Affine, Error> {
        self.star(&other)
    }
}

impl BitAnd for Affine {
    type Output = Result<Affine, Error>;
    fn bitand(self, other: Affine) -> Result<Affine, Error> {
        self.conj(&other)
    }
}

impl BitAnd<&'_ Affine> for Affine {
    type Output = Result<Affine, Error>;
    fn bitand(self, other: &Affine) -> Result<Affine, Error> {
        self.conj(other)
    }
}

impl BitAnd for &'_ Affine {
    type Output = Result<Affine, Error>;
    fn bitand(self, other: &Affine) -> Result<Affine, Error> {
        self.conj(other)
    }
}

impl BitAnd<Affine> for &'_ Affine {
    type Output = Result<Affine, Error>;
    fn bitand(self, other: Affine) -> Result<Affine, Error> {
        self.conj(&other)
    }
}
