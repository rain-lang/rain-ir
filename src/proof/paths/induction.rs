/*!
Path induction
*/
use super::*;

/// Path induction over a type or kind
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PathInd {
    /// The base types over which path induction is being performed
    base_tys: TyArr,
    /// The target kind over which path induction is to be performed
    target: KindId,
    /// The type of this instance of path induction
    ///
    /// This type's region must *always* be equal to this value's region, so no region pointer is necessary
    ty: VarId<Pi>,
}

impl PathInd {
    /// Create a new instance of path induction with a given base type
    pub fn try_new(base_tys: TyArr, target: KindId) -> Result<PathInd, Error> {
        let family_ty = Self::compute_family_ty(base_tys.clone(), target.clone())?.into_var();
        let ty = Self::ty_over_base_helper(base_tys.clone(), family_ty)?.into_var();
        Ok(PathInd {
            base_tys,
            target,
            ty,
        })
    }
    /// Get the type of path induction for a given base type
    pub fn compute_ty(base_ty: TyArr, target: KindId) -> Result<Pi, Error> {
        let family_ty = Self::compute_family_ty(base_ty.clone(), target)?.into_var();
        Self::ty_over_base_helper(base_ty, family_ty)
    }
    /// Get the type of families for an instance of path induction with a given base type
    pub fn compute_family_ty(base_tys: TyArr, target: KindId) -> Result<Pi, Error> {
        let left_region = Region::minimal(base_tys.clone())?;
        let right_region =
            Region::with(base_tys, left_region.clone()).expect("Right region is valid");
        let ids = left_region
            .params()
            .zip(right_region.params())
            .map(|(x, y)| {
                Id::try_new(x.into_val(), y.into_val())
                    .expect("Identity type is valid for same-type pairs")
                    .into_ty()
            });
        let id_region =
            Region::with(ids.collect(), right_region.clone()).expect("Identity region is valid");
        let target_pi = Pi::try_new(target.into_ty(), id_region).expect("Target pi");
        let right_pi = Pi::try_new(target_pi.into_ty(), right_region).expect("Right pi");
        Ok(Pi::try_new(right_pi.into_ty(), left_region).expect("Left pi"))
    }
    /// Get the type of loops for an instance of path induction with a given family
    pub fn compute_loop_ty(base_tys: TyArr, family: &ValId) -> Result<Pi, Error> {
        let arity = base_tys.len();
        let unary_region =
            Region::minimal(base_tys).expect("Single-parameter minimal region is always valid");
        let mut params = Vec::with_capacity(3 * arity);
        for param in unary_region.params() {
            params.push(param.into_val())
        }
        for ix in 0..arity {
            params.push(params[ix].clone())
        }
        for ix in 0..arity {
            params.push(Id::refl(params[ix].clone()).into_val())
        }
        let application = family
            .applied(&params[..])?
            .try_into_ty()
            .map_err(|_| Error::NotATypeError)?;
        Pi::try_new(application, unary_region)
    }
    /// Get the type of path induction for a given base type given the family type
    fn ty_over_base_helper(base_tys: TyArr, family_ty: VarId<Pi>) -> Result<Pi, Error> {
        let arity = base_tys.len();
        let family_region = Region::minimal(once(family_ty.into_ty()).collect())
            .expect("Single parameter minimal region is always valid");
        let family = family_region
            .param(0)
            .expect("Family region has first parameter")
            .into_val();
        let loop_ty = Self::compute_loop_ty(base_tys.clone(), &family).expect("Valid loop type");
        let loop_region = Region::with(once(loop_ty.into_ty()).collect(), family_region.clone())
            .expect("Loop region is valid");
        let left_region =
            Region::with(base_tys.clone(), loop_region.clone()).expect("Left region is valid");
        let right_region =
            Region::with(base_tys, left_region.clone()).expect("Right region is valid");
        let ids = left_region
            .params()
            .zip(right_region.params())
            .map(|(x, y)| {
                Id::try_new(x.into_val(), y.into_val())
                    .expect("Identity type is valid for same-type pairs")
                    .into_ty()
            });
        let id_region =
            Region::with(ids.collect(), right_region.clone()).expect("Identity region is valid");
        let mut params = Vec::with_capacity(3 * arity);
        for param in left_region.params() {
            params.push(param.into_val())
        }
        for param in right_region.params() {
            params.push(param.into_val())
        }
        for param in id_region.params() {
            params.push(param.into_val())
        }
        let application = family
            .applied(&params[..])
            .expect("Valid application of type family")
            .try_into_ty()
            .expect("Application of type family is a type");
        let specific_family_instantiation = Pi::try_new(application, id_region)
            .expect("Specific family instantiation is valid")
            .into_ty();
        let right_family_instantiation = Pi::try_new(specific_family_instantiation, right_region)
            .expect("Right family instantiation is valid")
            .into_ty();
        let left_family_instantiation = Pi::try_new(right_family_instantiation, left_region)
            .expect("Left family instantiation is valid")
            .into_ty();
        let loop_instantiation = Pi::try_new(left_family_instantiation, loop_region)
            .expect("Loop instantiation is valid")
            .into_ty();
        Ok(Pi::try_new(loop_instantiation, family_region).expect("Family instantiation is valid"))
    }
}

impl Typed for PathInd {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
}

impl Regional for PathInd {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.ty.region()
    }
}

impl Apply for PathInd {}

impl Substitute for PathInd {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<PathInd, Error> {
        let target = self.target.substitute_kind(ctx)?;
        let base_tys: Result<Vec<_>, _> = self
            .base_tys
            .iter()
            .map(|ty| ty.substitute_ty(ctx))
            .collect();
        let ty = self
            .ty
            .substitute(ctx)?
            .try_into()
            .map_err(|_| Error::InvalidSubKind)?;
        Ok(PathInd {
            target,
            base_tys: base_tys?.into(),
            ty,
        })
    }
}

substitute_to_valid!(PathInd);

enum_convert! {
    impl InjectionRef<ValueEnum> for PathInd {}
    impl TryFrom<NormalValue> for PathInd { as ValueEnum, }
    impl TryFromRef<NormalValue> for PathInd { as ValueEnum, }
}

impl Value for PathInd {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!("Invalid dependency {} for path induction", ix)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::PathInd(self))
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::PathInd(self)
    }
}

impl From<PathInd> for NormalValue {
    #[inline]
    fn from(path: PathInd) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::PathInd(path))
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for PathInd {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(path induction prettyprinting unimplemented)")
        }
    }
}
