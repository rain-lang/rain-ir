/*!
Relevant lifetimes
*/
use super::*;
use im::HashMap;

/// The data describing a purely relevant lifetime
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ReleventData {
    /// The relevant data
    data: HashMap<Color, Relevant>,
    /// Whether this data, taken together, is relevant
    relevant: bool,
}

/// A relevant lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Relevant {
    /// A completely used relevant object
    Used,
    //TODO: elemental relevance
}

impl Relevant {
    /// Take the separating conjunction of this lifetime with another
    pub fn star(&self, _other: &Relevant) -> Result<Relevant, Error> {
        Ok(Relevant::Used)
    }
    /// Take the conjunction of this lifetime with another
    pub fn conj(&self, _other: &Relevant) -> Result<Option<Relevant>, Error> {
        Ok(None)
    }
}

impl Mul for Relevant {
    type Output = Result<Relevant, Error>;
    fn mul(self, other: Relevant) -> Result<Relevant, Error> {
        self.star(&other)
    }
}

impl Mul<&'_ Relevant> for Relevant {
    type Output = Result<Relevant, Error>;
    fn mul(self, other: &Relevant) -> Result<Relevant, Error> {
        self.star(other)
    }
}

impl Mul for &'_ Relevant {
    type Output = Result<Relevant, Error>;
    fn mul(self, other: &Relevant) -> Result<Relevant, Error> {
        self.star(other)
    }
}

impl Mul<Relevant> for &'_ Relevant {
    type Output = Result<Relevant, Error>;
    fn mul(self, other: Relevant) -> Result<Relevant, Error> {
        self.star(&other)
    }
}

impl BitAnd for Relevant {
    type Output = Result<Option<Relevant>, Error>;
    fn bitand(self, other: Relevant) -> Result<Option<Relevant>, Error> {
        self.conj(&other)
    }
}

impl BitAnd<&'_ Relevant> for Relevant {
    type Output = Result<Option<Relevant>, Error>;
    fn bitand(self, other: &Relevant) -> Result<Option<Relevant>, Error> {
        self.conj(other)
    }
}

impl BitAnd for &'_ Relevant {
    type Output = Result<Option<Relevant>, Error>;
    fn bitand(self, other: &Relevant) -> Result<Option<Relevant>, Error> {
        self.conj(other)
    }
}

impl BitAnd<Relevant> for &'_ Relevant {
    type Output = Result<Option<Relevant>, Error>;
    fn bitand(self, other: Relevant) -> Result<Option<Relevant>, Error> {
        self.conj(&other)
    }
}
