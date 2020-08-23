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

/// The non-dependent applicativity axiom
///
/// NOTE: applicativity of dependent functions is not yet supported, as we do not yet support transport along types.
///
/// We also do not yet have a family of non-dependent applicativity axioms, as we first need a supported way to pass a TyArr at all.
///
/// TODO: this should not be a primitive value, but rather a descriptor for a primitive value to be constructed
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ApConst {
    /// The type of functions being applied
    ap_ty: VarId<Pi>,
    /// The particular function being applied, if any
    func: Option<ValId>,
}

impl ApConst {
    // === Construction ===

    /// Create a new instance of the applicativity axiom for a pi type
    #[inline]
    pub fn try_new_pi(ap_ty: VarId<Pi>) -> ApConst {
        ApConst { ap_ty, func: None }
    }
    /// Create a new instance of the applicativity axiom for a given function
    #[inline]
    pub fn try_new_fn(ap_ty: VarId<Pi>, param_fn: ValId) -> Result<ApConst, Error> {
        if param_fn.ty() != ap_ty {
            return Err(Error::TypeMismatch);
        }
        Ok(ApConst { ap_ty, func: None })
    }

    // === Manipulation ===

    /// Get the base type of this instance of the applicativity axiom
    #[inline]
    pub fn ap_ty(&self) -> &VarId<Pi> {
        &self.ap_ty
    }
    /// Get the particular function being applied by this instance, if any
    #[inline]
    pub fn func(&self) -> Option<&ValId> {
        self.func.as_ref()
    }
    /// Set the function being applied by this instance, returning the old one if any
    ///
    /// Leave this value unchanged and return an error if the function is incompatible with this instance.
    #[inline]
    pub fn set_func(&mut self, mut func: Option<ValId>) -> Result<Option<ValId>, Error> {
        if let Some(func) = &func {
            if func.ty() != self.ap_ty {
                return Err(Error::TypeMismatch);
            }
        }
        std::mem::swap(&mut func, &mut self.func);
        Ok(func)
    }

    // === Value constructon ===

    /// Construct a `ValId` corresponding to a proof of path induction for the given instance of `ApConst`
    #[inline]
    pub fn into_val(self) -> ValId {
        if let Some(param_fn) = self.func {
            let domain = self.ap_ty.param_tys().clone();
            Self::prove_for_func(param_fn, domain)
                .expect("Transforming a valid ApConst instance to a `ValId` should always succeed!")
        } else {
            Self::prove_over(self.ap_ty)
        }
    }

    /// Construct a `ValId` corresponding to a proof of applicativity for a given (fixed) function and domain
    pub fn prove_for_func(param_fn: ValId, domain: TyArr) -> Result<ValId, Error> {
        unimplemented!(
            "Prove apconst for function {} over domain {:?} (unverified)",
            param_fn,
            domain
        )
    }

    /// Construct a `ValId` corresponding to a proof of applicativity for any function of a given type
    pub fn prove_over(ap_ty: VarId<Pi>) -> ValId {
        unimplemented!("Prove apconst for function type {}", ap_ty)
    }

    // === Type construction ===

    /// Compute the type of this instance of the applicativity axiom as a `VarId<Pi>`
    #[inline]
    pub fn compute_ty(&self) -> VarId<Pi> {
        self.compute_pi().into_var()
    }
    /// Compute the pi type of this instance of the applicativity axiom: warning, slow!
    #[inline]
    pub fn compute_pi(&self) -> Pi {
        let result_pi = if let Some(param_fn) = &self.func {
            Self::fn_ty(&self.ap_ty, param_fn.clone())
        } else {
            Self::pi_ty(self.ap_ty.clone())
        };
        result_pi.expect("Constructing an ApConst instance is always valid")
    }
    /// Get the pi type corresponding to an instance of this axiom for a given function
    pub fn fn_ty(ap_ty: &VarId<Pi>, param_fn: ValId) -> Result<Pi, Error> {
        if param_fn.ty() != *ap_ty {
            //TODO: subtyping?
            return Err(Error::TypeMismatch);
        }
        let domain = ap_ty.param_tys().clone();
        Self::fn_ty_helper(param_fn, domain)
    }
    /// Get the pi type corresponding to an instance of this axiom for a given function type
    pub fn pi_ty(ap_ty: VarId<Pi>) -> Result<Pi, Error> {
        let domain = ap_ty.param_tys().clone();
        let ap_ty_region = ap_ty.clone_region();
        let pi_region = Region::with(once(ap_ty.into_ty()).collect(), ap_ty_region)
            .expect("ap_ty lies in it's own region...");
        let param_fn = pi_region
            .param(0)
            .expect("Pi region has exactly one parameter")
            .into_val();
        let param_pi = Self::fn_ty_helper(param_fn, domain)?;
        Ok(Pi::try_new(param_pi.into_ty(), pi_region).expect("Final pi is valid"))
    }
    fn fn_ty_helper(param_fn: ValId, domain: TyArr) -> Result<Pi, Error> {
        let no_params = domain.len();
        let left_region = Region::with(domain.clone(), param_fn.clone_region())
            .expect("domain lies in ap_ty's region");
        let right_region = Region::with(domain, left_region.clone_region())
            .expect("domain lies in ap_ty's region");
        let mut left_params = Vec::with_capacity(no_params);
        let mut right_params = Vec::with_capacity(no_params);
        let identity_region =
            Self::construct_identity(&left_region, &right_region, |left, right| {
                left_params.push(left.clone());
                right_params.push(right.clone())
            })
            .expect("Constructing identity region works");
        let left_ap = param_fn.applied(&left_params[..])?;
        let right_ap = param_fn.applied(&right_params[..])?;
        let result_id = Id::try_new(left_ap, right_ap)?;
        let arrow_pi =
            Pi::try_new(result_id.into_ty(), identity_region).expect("Arrow pi is valid");
        let right_pi = Pi::try_new(arrow_pi.into_ty(), right_region).expect("Right pi is valid");
        Ok(Pi::try_new(right_pi.into_ty(), left_region).expect("Left pi is valid"))
    }

    // === Helpers

    /// Construct an identity region over a left and right region, while calling a callback on the generated left and right parameter `ValId`s
    pub fn construct_identity<F>(
        left: &Region,
        right: &Region,
        mut callback: F,
    ) -> Result<Region, Error>
    where
        F: FnMut(&ValId, &ValId),
    {
        let left_len = left.len();
        let right_len = right.len();
        if left_len != right_len {
            return Err(Error::TooManyArgs);
        }
        let mut identity_params = Vec::with_capacity(left_len);
        for (left, right) in left.params().zip(right.params()) {
            let left = left.into_val();
            let right = right.into_val();
            callback(&left, &right);
            identity_params.push(Id::try_new(left, right)?.into_ty());
        }
        Region::with(identity_params.into(), right.clone())
    }

    /// Construct a left region, right region, and identity region for a domain over an (optional) base region
    /// 
    /// If the base region is null, constructs a minimal region
    pub fn construct_left_right_identity(
        domain: TyArr,
        base: Option<Region>,
    ) -> Result<(Region, Region, Region), Error> {
        unimplemented!()
    }
}

impl From<ApConst> for ValId {
    #[inline]
    fn from(ap_const: ApConst) -> ValId {
        ap_const.into_val()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::primitive::logical::binary_ty;
    use crate::value::Value;

    fn manually_construct_binary_happly() -> VarId<Pi> {
        let binary_ty = binary_ty();
        let binary_region =
            Region::with(once(binary_ty.clone().into_ty()).collect(), Region::NULL).unwrap();
        let operator = binary_region.param(0).unwrap().into_val();
        let left_region =
            Region::with(binary_ty.param_tys().clone(), binary_region.clone()).unwrap();
        let right_region =
            Region::with(binary_ty.param_tys().clone(), left_region.clone()).unwrap();
        let mut identity_params = Vec::with_capacity(2);
        let mut left_params = Vec::with_capacity(2);
        let mut right_params = Vec::with_capacity(2);
        for (left, right) in left_region.params().zip(right_region.params()) {
            let left = left.into_val();
            let right = right.into_val();
            left_params.push(left.clone());
            right_params.push(right.clone());
            identity_params.push(Id::try_new(left, right).unwrap().into_ty());
        }
        let identity_region = Region::with(identity_params.into(), right_region.clone()).unwrap();
        let left_ap = operator.applied(&left_params[..]).unwrap();
        let right_ap = operator.applied(&right_params[..]).unwrap();
        let result_id = Id::try_new(left_ap, right_ap).unwrap();
        let arrow_pi =
            Pi::try_new(result_id.into_ty(), identity_region).expect("Arrow pi is valid");
        let right_pi = Pi::try_new(arrow_pi.into_ty(), right_region).expect("Right pi is valid");
        let left_pi = Pi::try_new(right_pi.into_ty(), left_region).expect("Left pi is valid");
        Pi::try_new(left_pi.into_ty(), binary_region)
            .expect("Binary operation application type is valid")
            .into_var()
    }

    #[test]
    fn happly_helpers() {
        let binary_ty = binary_ty();
        let manual_ap_type = manually_construct_binary_happly();
        let ap_const = ApConst::try_new_pi(binary_ty);
        let ap_type = ap_const.compute_ty();
        assert_eq!(ap_type, manual_ap_type);
    }
}
