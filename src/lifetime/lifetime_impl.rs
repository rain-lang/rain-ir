/*!
The implementation of `rain` lifetimes
*/

use super::*;

impl PartialEq for Lifetime {
    fn eq(&self, other: &Lifetime) -> bool {
        self.data_ptr() == other.data_ptr()
    }
}

impl Hash for Lifetime {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.data_ptr(), hasher)
    }
}

impl PartialEq for LifetimeBorrow<'_> {
    fn eq(&self, other: &LifetimeBorrow) -> bool {
        self.data_ptr() == other.data_ptr()
    }
}

impl PartialEq<Lifetime> for LifetimeBorrow<'_> {
    fn eq(&self, other: &Lifetime) -> bool {
        *self == other.borrow_lifetime()
    }
}

impl PartialEq<LifetimeBorrow<'_>> for Lifetime {
    fn eq(&self, other: &LifetimeBorrow) -> bool {
        self.borrow_lifetime() == *other
    }
}

impl Hash for LifetimeBorrow<'_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.data_ptr(), hasher)
    }
}

impl Add for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn add(self, other: Lifetime) -> Result<Lifetime, Error> {
        self.disj(&other)
    }
}

impl Add<&'_ Lifetime> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn add(self, other: &Lifetime) -> Result<Lifetime, Error> {
        self.disj(other)
    }
}

impl Add<LifetimeBorrow<'_>> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn add(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.add(other.as_lifetime())
    }
}

impl Add<Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn add(self, other: Lifetime) -> Result<Lifetime, Error> {
        other.add(self)
    }
}

impl Add<&'_ Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn add(self, other: &Lifetime) -> Result<Lifetime, Error> {
        self.disj(other)
    }
}

impl Add<LifetimeBorrow<'_>> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn add(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.add(other.as_lifetime())
    }
}

impl Add<Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn add(self, other: Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().add(other)
    }
}

impl Add<&'_ Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn add(self, other: &Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().add(other)
    }
}

impl Add<LifetimeBorrow<'_>> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn add(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.as_lifetime().add(other.as_lifetime())
    }
}

impl Mul for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: Lifetime) -> Result<Lifetime, Error> {
        self.sep_conj(&other)
    }
}

impl Mul<&'_ Lifetime> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: &Lifetime) -> Result<Lifetime, Error> {
        self.sep_conj(other)
    }
}

impl Mul<LifetimeBorrow<'_>> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.sep_conj(&other)
    }
}

impl Mul<Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: Lifetime) -> Result<Lifetime, Error> {
        self.sep_conj(&other)
    }
}

impl Mul<&'_ Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: &Lifetime) -> Result<Lifetime, Error> {
        self.sep_conj(other)
    }
}

impl Mul<LifetimeBorrow<'_>> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.mul(other.as_lifetime())
    }
}

impl Mul<Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().mul(other)
    }
}

impl Mul<&'_ Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: &Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().mul(other)
    }
}

impl Mul<LifetimeBorrow<'_>> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.as_lifetime().mul(other.as_lifetime())
    }
}

impl Regional for Lifetime {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.deref().region()
    }
}

impl From<LifetimeData> for Lifetime {
    #[inline]
    fn from(data: LifetimeData) -> Lifetime {
        Lifetime::new(data)
    }
}

impl From<Option<Region>> for Lifetime {
    #[inline]
    fn from(region: Option<Region>) -> Lifetime {
        region.map(Lifetime::from).unwrap_or(Lifetime(None))
    }
}

impl From<Region> for Lifetime {
    #[inline]
    fn from(region: Region) -> Lifetime {
        Lifetime::new(LifetimeData::from(region))
    }
}

impl PartialOrd for Lifetime {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes.
    */
    fn partial_cmp(&self, other: &Lifetime) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialOrd<LifetimeBorrow<'_>> for Lifetime {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes.
    */
    fn partial_cmp(&self, other: &LifetimeBorrow<'_>) -> Option<Ordering> {
        self.partial_cmp(other.deref())
    }
}

impl PartialOrd for LifetimeBorrow<'_> {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes
    */
    fn partial_cmp(&self, other: &LifetimeBorrow<'_>) -> Option<Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl PartialOrd<Lifetime> for LifetimeBorrow<'_> {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes.
    */
    fn partial_cmp(&self, other: &Lifetime) -> Option<Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl Regional for LifetimeBorrow<'_> {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.deref().region()
    }
}
