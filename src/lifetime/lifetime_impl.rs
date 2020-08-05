/*!
The implementation of `rain` lifetimes
*/

use super::*;

impl PartialEq for Lifetime {
    fn eq(&self, other: &Lifetime) -> bool {
        let self_ptr = self.deref() as *const _;
        let other_ptr = other.deref() as *const _;
        self_ptr == other_ptr
    }
}

impl Hash for Lifetime {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

impl Deref for Lifetime {
    type Target = LifetimeData;
    fn deref(&self) -> &LifetimeData {
        if let Some(ptr) = &self.0 {
            &ptr
        } else {
            &STATIC_LIFETIME
        }
    }
}

impl PartialEq for LifetimeBorrow<'_> {
    fn eq(&self, other: &LifetimeBorrow) -> bool {
        let self_ptr = self.deref() as *const _;
        let other_ptr = other.deref() as *const _;
        self_ptr == other_ptr
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

impl Deref for LifetimeBorrow<'_> {
    type Target = LifetimeData;
    fn deref(&self) -> &LifetimeData {
        if let Some(ptr) = &self.0 {
            &ptr
        } else {
            &STATIC_LIFETIME
        }
    }
}

impl Hash for LifetimeBorrow<'_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}


impl BitAnd for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: Lifetime) -> Result<Lifetime, Error> {
        if self == other || other.is_static() {
            return Ok(self);
        }
        if self.is_static() {
            return Ok(other);
        }
        self.deref().conj(other.deref()).map(Lifetime::new)
    }
}

impl BitAnd<&'_ Lifetime> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: &Lifetime) -> Result<Lifetime, Error> {
        if self == *other || other.is_static() {
            return Ok(self);
        }
        if self.is_static() {
            return Ok(other.clone());
        }
        self.deref().conj(other.deref()).map(Lifetime::new)
    }
}

impl BitAnd<LifetimeBorrow<'_>> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.bitand(other.as_lifetime())
    }
}

impl BitAnd<Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: Lifetime) -> Result<Lifetime, Error> {
        other.bitand(self)
    }
}

impl BitAnd<&'_ Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: &Lifetime) -> Result<Lifetime, Error> {
        other.join(self)
    }
}

impl BitAnd<LifetimeBorrow<'_>> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.bitand(other.as_lifetime())
    }
}

impl BitAnd<Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().bitand(other)
    }
}

impl BitAnd<&'_ Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: &Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().bitand(other)
    }
}

impl BitAnd<LifetimeBorrow<'_>> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.as_lifetime().bitand(other.as_lifetime())
    }
}

impl Mul for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: Lifetime) -> Result<Lifetime, Error> {
        if self == other || other.is_static() {
            return Ok(self);
        }
        if self.is_static() {
            return Ok(other);
        }
        self.deref().conj(other.deref()).map(Lifetime::new)
    }
}

impl Mul<&'_ Lifetime> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: &Lifetime) -> Result<Lifetime, Error> {
        if self == *other || other.is_static() {
            return Ok(self);
        }
        if self.is_static() {
            return Ok(other.clone());
        }
        self.deref().conj(other.deref()).map(Lifetime::new)
    }
}

impl Mul<LifetimeBorrow<'_>> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.mul(other.as_lifetime())
    }
}

impl Mul<Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: Lifetime) -> Result<Lifetime, Error> {
        other.mul(self)
    }
}

impl Mul<&'_ Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: &Lifetime) -> Result<Lifetime, Error> {
        other.join(self)
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
        self.deref().partial_cmp(other.deref())
    }
}

impl PartialOrd<LifetimeBorrow<'_>> for Lifetime {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes.
    */
    fn partial_cmp(&self, other: &LifetimeBorrow<'_>) -> Option<Ordering> {
        self.deref().partial_cmp(other.deref())
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
        self.deref().partial_cmp(other.deref())
    }
}

impl Regional for LifetimeBorrow<'_> {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.deref().region()
    }
}