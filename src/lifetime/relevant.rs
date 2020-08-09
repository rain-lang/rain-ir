/*!
Relevant lifetimes
*/
use super::*;
use fxhash::{FxBuildHasher, FxHashMap};
use im::{hashmap::Entry, HashMap};

/// The data describing a purely relevant lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct RelevantData {
    /// The relevant data
    pub(super) data: HashMap<Color, Relevant, FxBuildHasher>,
}

impl RelevantData {
    /// Create a relevant lifetime from a single obligation
    pub fn unit(color: Color, relevance: Relevant) -> RelevantData {
        let mut data = HashMap::default();
        data.insert(color, relevance);
        RelevantData { data }
    }
    /// Create a relevant lifetime only using a given color
    pub fn uses(color: Color) -> RelevantData {
        Self::unit(color, Relevant::Used)
    }
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
    pub fn disj(&self, other: &RelevantData) -> RelevantData {
        let mut data = HashMap::new_from(&self.data);
        for (color, relevance) in self.data.iter() {
            if let Some(other_relevance) = other.data.get(color) {
                if let Some(new_relevance) = relevance + other_relevance {
                    data.insert(color.clone(), new_relevance);
                }
            }
        }
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
    /// Perform a color-mapping of this lifetime
    ///
    /// Leaves this lifetime in an undetermined but valid state upon failure
    #[inline]
    pub fn color_map<'a, F>(&mut self, mut color_map: F, depth: usize) -> Result<(), Error>
    where
        F: FnMut(&Color) -> Option<&'a Lifetime>,
    {
        let mut relevant: FxHashMap<Color, Relevant> = FxHashMap::default();
        self.data.retain(|key, _value| {
            use std::collections::hash_map::Entry;
            if key.depth() < depth {
                return true;
            }
            if let Some(lifetime) = color_map(key).map(Lifetime::data).flatten() {
                for (color, relevance) in lifetime.relevant().data.iter() {
                    //TODO: relative relevance!
                    match relevant.entry(color.clone()) {
                        Entry::Occupied(mut o) => {
                            *o.get_mut() = o.get() * relevance;
                        }
                        Entry::Vacant(v) => {
                            v.insert(relevance.clone());
                        }
                    }
                }
                false
            } else {
                unimplemented!("Single-color escape")
            }
        });
        for (color, relevance) in relevant.into_iter() {
            match self.data.entry(color) {
                Entry::Occupied(mut o) => {
                    let new_relevance = o.get() * relevance;
                    if new_relevance != *o.get() {
                        *o.get_mut() = new_relevance;
                    }
                }
                Entry::Vacant(v) => {
                    v.insert(relevance);
                }
            }
        }
        Ok(())
    }
}

impl PartialOrd for RelevantData {
    fn partial_cmp(&self, other: &RelevantData) -> Option<Ordering> {
        fn subset_cmp(left: &RelevantData, right: &RelevantData) -> Option<Ordering> {
            use Ordering::*;
            let mut strict_sub = false;
            for (color, affinity) in left.data.iter() {
                if let Some(other_affinity) = right.data.get(color) {
                    match affinity.partial_cmp(other_affinity)? {
                        Greater => strict_sub = true,
                        Equal => {}
                        Less => return None,
                    }
                } else {
                    return None;
                }
            }
            if strict_sub || left.len() < right.len() {
                Some(Less)
            } else {
                Some(Equal)
            }
        }
        if self.len() <= other.len() {
            subset_cmp(self, other)
        } else {
            subset_cmp(other, self).map(Ordering::reverse)
        }
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
        self.disj(&other)
    }
}

impl Add<&'_ RelevantData> for RelevantData {
    type Output = RelevantData;
    #[inline]
    fn add(self, other: &RelevantData) -> RelevantData {
        self.disj(other)
    }
}

impl Add for &'_ RelevantData {
    type Output = RelevantData;
    #[inline]
    fn add(self, other: &RelevantData) -> RelevantData {
        self.disj(other)
    }
}

impl Add<RelevantData> for &'_ RelevantData {
    type Output = RelevantData;
    #[inline]
    fn add(self, other: RelevantData) -> RelevantData {
        self.disj(&other)
    }
}

/// A relevant lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Relevant {
    /// A completely used relevant object
    Used,
    //TODO: elemental relevance
}

impl PartialOrd for Relevant {
    #[inline]
    fn partial_cmp(&self, _other: &Relevant) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
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
