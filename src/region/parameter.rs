/*!
Parameters to a `rain` region
*/
use super::{Region, RegionBorrow, Regional};
use crate::enum_convert;
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

enum_convert! {
    impl InjectionRef<ValueEnum> for Parameter {}
    impl TryFrom<NormalValue> for Parameter { as ValueEnum, }
    impl TryFromRef<NormalValue> for Parameter { as ValueEnum, }
}

impl Parameter {
    /**
     Reference the `ix`th parameter of the given region. Return `Err` if the parameter is out of bounds.

    # Examples
    Trying to make a parameter out of bounds returns `Err`:
    ```rust
    use rain_ir::region::{Region, RegionData, Parameter};
    let empty_region = Region::with_parent(Region::default());
    assert_eq!(Parameter::try_new(empty_region, 1), Err(()));
    ```
    */
    #[inline]
    pub fn try_new(region: Region, ix: usize) -> Result<Parameter, ()> {
        if ix >= region.len() {
            Err(())
        } else {
            let lifetime = Lifetime::param(region.clone(), ix);
            Ok(Parameter {
                region,
                lifetime,
                ix,
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
        args: &'a [ValId],
        _inline: bool,
        _ctx: Option<&mut EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        if args.is_empty() {
            return Ok(Application::Success(args, self.clone().into()));
        }
        match self.ty().as_enum() {
            ValueEnum::Pi(p) => unimplemented!("Pi type parameters: parameter = {}: {}", self, p),
            ValueEnum::Product(p) => {
                if args.len() > 1 {
                    unimplemented!("Multi-tuple application: blocking on ApplyTy")
                }
                let ix = match args[0].as_enum() {
                    ValueEnum::Index(ix) => {
                        if ix.get_ty().0 == p.len() as u128 {
                            ix.ix()
                        } else {
                            return Err(Error::TupleLengthMismatch);
                        }
                    }
                    ValueEnum::Parameter(pi) => unimplemented!(
                        "Parameter {} indexing parameter product {}: {}",
                        pi,
                        self,
                        p
                    ),
                    _ => return Err(Error::TypeMismatch),
                };
                let lt = p.lifetime().clone_lifetime(); // TODO: this
                let ty = p[ix as usize].clone();
                Ok(Application::Stop(lt, ty))
            }
            _ => Err(Error::NotAFunction),
        }
    }
}

impl From<Parameter> for NormalValue {
    fn from(param: Parameter) -> NormalValue {
        NormalValue(ValueEnum::Parameter(param))
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
