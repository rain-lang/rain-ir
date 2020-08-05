/*!
Relevant lifetimes
*/
use super::*;
use fxhash::FxBuildHasher;
use im::{hashmap::Entry, HashMap};

/// The data describing a purely relevant lifetime
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct RelevantData {
    /// The relevant data
    data: HashMap<Color, Relevant, FxBuildHasher>,
}

impl RelevantData {
    /// Take the separating conjunction of this lifetime with another
    pub fn sep_conj(&mut self, other: &RelevantData) {
        for (color, relevance) in other.data.iter() {
            match self.data.entry(color.clone()) {
                Entry::Occupied(o) => {
                    let other_relevance = o.get();
                    let new_relevance = other_relevance.sep_conj(other_relevance);
                    if new_relevance != *relevance {
                        o.insert(new_relevance);
                    }
                }
                Entry::Vacant(v) => {
                    v.insert(relevance.clone());
                }
            }
        }
    }
    /// Take the conjunction of this lifetime with another
    pub fn conj(self, other: RelevantData) -> RelevantData {
        let data = self
            .data
            .symmetric_difference_with(other.data, BitAnd::bitand);
        RelevantData { data }
    }
}

impl Mul for RelevantData {
    type Output = RelevantData;
    #[inline]
    fn mul(self, other: RelevantData) -> RelevantData {
        //TODO: think about this...
        self * &other
    }
}

impl Mul<&'_ RelevantData> for RelevantData {
    type Output = RelevantData;
    #[inline]
    fn mul(self, other: &RelevantData) -> RelevantData {
        other * self
    }
}

impl Mul for &'_ RelevantData {
    type Output = RelevantData;
    #[inline]
    fn mul(self, other: &RelevantData) -> RelevantData {
        self * other.clone()
    }
}

impl Mul<RelevantData> for &'_ RelevantData {
    type Output = RelevantData;
    #[inline]
    fn mul(self, mut other: RelevantData) -> RelevantData {
        other.sep_conj(self);
        other
    }
}

impl BitAnd for RelevantData {
    type Output = RelevantData;
    #[inline]
    fn bitand(self, other: RelevantData) -> RelevantData {
        self.conj(other)
    }
}

impl BitAnd<&'_ RelevantData> for RelevantData {
    type Output = RelevantData;
    #[inline]
    fn bitand(self, other: &RelevantData) -> RelevantData {
        self.conj(other.clone())
    }
}

impl BitAnd for &'_ RelevantData {
    type Output = RelevantData;
    #[inline]
    fn bitand(self, other: &RelevantData) -> RelevantData {
        self.conj(other.clone())
    }
}

impl BitAnd<RelevantData> for &'_ RelevantData {
    type Output = RelevantData;
    #[inline]
    fn bitand(self, other: RelevantData) -> RelevantData {
        self.clone().conj(other)
    }
}


/// A relevant lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Relevant {
    /// A completely used relevant object
    Used,
    //TODO: elemental relevance
}

impl Relevant {
    pub fn sep_conj(&self, _other: &Relevant) -> Relevant {
        Relevant::Used
    }
    pub fn conj(&self, _other: &Relevant) -> Option<Relevant> {
        Some(Relevant::Used)
    }
}

impl Mul for Relevant {
    type Output = Relevant;
    fn mul(self, other: Relevant) -> Relevant {
        self.sep_conj(&other)
    }
}

impl Mul<&'_ Relevant> for Relevant {
    type Output = Relevant;
    fn mul(self, other: &Relevant) -> Relevant {
        self.sep_conj(other)
    }
}

impl Mul for &'_ Relevant {
    type Output = Relevant;
    fn mul(self, other: &Relevant) -> Relevant {
        self.sep_conj(other)
    }
}

impl Mul<Relevant> for &'_ Relevant {
    type Output = Relevant;
    fn mul(self, other: Relevant) -> Relevant {
        self.sep_conj(&other)
    }
}

impl BitAnd for Relevant {
    type Output = Option<Relevant>;
    fn bitand(self, other: Relevant) -> Option<Relevant> {
        self.conj(&other)
    }
}

impl BitAnd<&'_ Relevant> for Relevant {
    type Output = Option<Relevant>;
    fn bitand(self, other: &Relevant) -> Option<Relevant> {
        self.conj(other)
    }
}

impl BitAnd for &'_ Relevant {
    type Output = Option<Relevant>;
    fn bitand(self, other: &Relevant) -> Option<Relevant> {
        self.conj(other)
    }
}

impl BitAnd<Relevant> for &'_ Relevant {
    type Output = Option<Relevant>;
    fn bitand(self, other: Relevant) -> Option<Relevant> {
        self.conj(&other)
    }
}
