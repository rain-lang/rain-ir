/*!
Parameters to a `rain` region
*/
use super::{Region, RegionBorrow, Regional};
use crate::eval::{Application, Apply, EvalCtx};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::typing::{Type, Typed};
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

impl Parameter {
    /**
     Reference the `ix`th parameter of the given region. Return `Err` if the parameter is out of bounds.

    # Examples
    Trying to make a parameter out of bounds returns `Err`:
    ```rust
    use rain_ir::region::{Region, RegionData, Parameter};
    let empty_region = Region::new(RegionData::new(Region::default()));
    assert_eq!(Parameter::try_new(empty_region, 1), Err(()));
    ```
    */
    #[inline]
    pub fn try_new(region: Region, ix: usize) -> Result<Parameter, ()> {
        if ix >= region.len() {
            Err(())
        } else {
            let lifetime = region.clone().into();
            Ok(Parameter {
                region,
                ix,
                lifetime,
            })
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
    fn region(&self) -> RegionBorrow {
        self.get_region().borrow_region()
    }
}

impl Typed for Parameter {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.region[self.ix].borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.ty().is_universe()
    }
}

impl Apply for Parameter {
    fn do_apply_in_ctx<'a>(
        &self,
        _args: &'a [ValId],
        _inline: bool,
        _ctx: Option<&mut EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        match self.ty().as_enum() {
            ValueEnum::Pi(p) => unimplemented!("Pi type parameters: parameter = {}: {}", self, p),
            ValueEnum::Product(p) => {
                unimplemented!("Product type parameters: parameter = {}: {}", self, p)
            }
            _ => Err(Error::NotAFunction),
        }
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
