/*!
Relevant lifetimes
*/
use super::*;
use fxhash::FxBuildHasher;
use im::{hashmap::Entry, HashMap};

/// The data describing a purely relevant lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct RelevantData {
    /// The relevant data
    data: HashMap<Color, Relevant, FxBuildHasher>,
}

impl RelevantData {
    /// Take the separating conjunction of this lifetime with another
    pub fn sep_conj(&mut self, other: &RelevantData) {
        for (color, relevance) in other.data.iter() {
            match self.data.entry(color.clone()) {
                Entry::Occupied(mut o) => {
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
    /// Take the disjunction of this lifetime with another
    pub fn disj(self, other: RelevantData) -> RelevantData {
        let data = self.data.symmetric_difference_with(other.data, Add::add);
        RelevantData { data }
    }
    /// Whether this lifetime is the static lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.data.is_empty()
    }
    /// Whether this lifetime contains any relevant data
    ///
    /// All purely relevant lifetimes are idempotent under separating conjunction, and
    /// treat the static lifetime as a zero element under disjunction
    #[inline]
    pub fn is_relevant(&self) -> bool {
        self.data.is_empty()
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

impl Add for RelevantData {
    type Output = RelevantData;
    #[inline]
    fn add(self, other: RelevantData) -> RelevantData {
        self.disj(other)
    }
}

impl Add<&'_ RelevantData> for RelevantData {
    type Output = RelevantData;
    #[inline]
    fn add(self, other: &RelevantData) -> RelevantData {
        self.disj(other.clone())
    }
}

impl Add for &'_ RelevantData {
    type Output = RelevantData;
    #[inline]
    fn add(self, other: &RelevantData) -> RelevantData {
        self.clone().disj(other.clone())
    }
}

impl Add<RelevantData> for &'_ RelevantData {
    type Output = RelevantData;
    #[inline]
    fn add(self, other: RelevantData) -> RelevantData {
        self.clone().disj(other)
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
    /// Take the separating conjunction of two relevant lifetimes
    pub fn sep_conj(&self, _other: &Relevant) -> Relevant {
        Relevant::Used
    }
    /// Take the disjunction of two relevant lifetimes
    pub fn disj(&self, _other: &Relevant) -> Option<Relevant> {
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

impl Add for Relevant {
    type Output = Option<Relevant>;
    fn add(self, other: Relevant) -> Option<Relevant> {
        self.disj(&other)
    }
}

impl Add<&'_ Relevant> for Relevant {
    type Output = Option<Relevant>;
    fn add(self, other: &Relevant) -> Option<Relevant> {
        self.disj(other)
    }
}

impl Add for &'_ Relevant {
    type Output = Option<Relevant>;
    fn add(self, other: &Relevant) -> Option<Relevant> {
        self.disj(other)
    }
}

impl Add<Relevant> for &'_ Relevant {
    type Output = Option<Relevant>;
    fn add(self, other: Relevant) -> Option<Relevant> {
        self.disj(&other)
    }
}
