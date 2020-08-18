/*!
Parameters to a `rain` region
*/
use super::{Region, RegionBorrow, Regional};
use crate::enum_convert;
use crate::eval::Apply;
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::typing::Typed;
use crate::value::{Error, NormalValue, TypeRef, ValId, Value, ValueData, ValueEnum};
use crate::{quick_pretty, trivial_substitute};

/**
A parameter to a `rain` region.

Note that the uniqueness of `ValId`s corresponding to a given parameter is enforced by the hash-consing algorithm.
*/
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parameter {
    /// The region this is a parameter for
    region: Region,
    /// The index of this parameter in the region's type vector
    ix: usize,
    /// The lifetime of this parameter
    lifetime: Lifetime,
}

quick_pretty!(Parameter, s, fmt => write!(fmt, "#parameter(depth={}, ix={})", s.depth(), s.ix()));
trivial_substitute!(Parameter);

enum_convert! {
    impl InjectionRef<ValueEnum> for Parameter {}
    impl TryFrom<NormalValue> for Parameter { as ValueEnum, }
    impl TryFromRef<NormalValue> for Parameter { as ValueEnum, }
}

impl Parameter {
    /**
    Create a parameter from a given region and index, when you know the region is in bounds.

    Note that this will *still* return an error if the parameter is out of bounds, *but* suffers from degraded performance in this case.
    */
    #[inline]
    pub fn try_new_inbounds(region: Region, ix: usize) -> Result<Parameter, Error> {
        let lifetime = Lifetime::param(region.clone(), ix)?;
        Ok(Parameter {
            region,
            lifetime,
            ix,
        })
    }
    /**
     Reference the `ix`th parameter of the given region. Return `Err` if the parameter is out of bounds.

    # Examples
    Trying to make a parameter out of bounds returns `Err`:
    ```rust
    use rain_ir::region::{Region, Parameter};
    use rain_ir::value::Error;
    let empty_region = Region::with_parent(None);
    assert_eq!(Parameter::try_new(empty_region, 1), Err(Error::InvalidParam));
    ```
    */
    #[inline]
    pub fn try_new(region: Region, ix: usize) -> Result<Parameter, Error> {
        if ix >= region.len() {
            Err(Error::InvalidParam)
        } else {
            Self::try_new_inbounds(region, ix)
        }
    }
    /**
     Reference the `ix`th parameter of the given region. Return `Err` if the parameter is out of bounds.

    # Examples
    Trying to make a parameter out of bounds returns `Err`:
    ```rust
    use rain_ir::region::{Region, Parameter};
    use rain_ir::value::Error;
    let empty_region = Region::with_parent(None);
    assert_eq!(Parameter::try_clone_new(&empty_region, 1), Err(Error::InvalidParam));
    ```
    */
    #[inline]
    pub fn try_clone_new(region: &Region, ix: usize) -> Result<Parameter, Error> {
        if ix >= region.len() {
            Err(Error::InvalidParam)
        } else {
            Self::try_new_inbounds(region.clone(), ix)
        }
    }
    /// Get the index of this parameter
    #[inline]
    pub fn ix(&self) -> usize {
        self.ix
    }
    /// Get this parameter's region
    #[inline]
    pub fn get_region(&self) -> &Region {
        &self.region
    }
}

impl Live for Parameter {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

impl Regional for Parameter {
    fn region(&self) -> Option<RegionBorrow> {
        Some(self.get_region().borrow_region())
    }
    fn depth(&self) -> usize {
        let depth = self.get_region().depth();
        debug_assert!(depth > 0);
        depth
    }
}

impl Typed for Parameter {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.region[self.ix].borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.ty().is_kind()
    }
    #[inline]
    fn is_kind(&self) -> bool {
        self.ty().ty().is_kind()
    }
}

impl Apply for Parameter {}

impl From<Parameter> for NormalValue {
    fn from(param: Parameter) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Parameter(param))
    }
}

impl Value for Parameter {
    fn no_deps(&self) -> usize {
        0
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!("Attempted to get dependency {} of parameter #{} of a region, but parameters have no deps!", ix, self.ix)
    }
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Parameter(self)
    }
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for Parameter {}
