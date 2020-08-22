/*!
Proofs of identity and equivalence.
*/
use crate::eval::Substitute;
use crate::eval::{Application, Apply, EvalCtx};
use crate::function::pi::Pi;
use crate::region::{Region, RegionBorrow, Regional};
use crate::typing::{Kind, Type, Typed};
use crate::value::{
    arr::TyArr, Error, KindId, NormalValue, TypeId, TypeRef, ValId, Value, ValueEnum, VarId,
};
use crate::{enum_convert, substitute_to_valid};
use std::convert::TryInto;
use std::iter::once;
//use either::Either;

/// The identity type family, either of a given type or in general
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IdFamily {
    /// The base type of this family, if any
    base_ty: Option<TypeId>,
    /// The type of this family
    ty: VarId<Pi>,
    /// The region of this family
    region: Region,
}

impl IdFamily {
    /// Get the constructor for all identity type families within a given kind
    pub fn universal(kind: KindId) -> IdFamily {
        let ty = Self::universal_pi(&kind).into_var();
        let region = kind.clone_region();
        IdFamily {
            ty,
            region,
            base_ty: None,
        }
    }
    /// Get a given identity type family
    pub fn family(base_ty: TypeId) -> IdFamily {
        let ty = Self::family_pi(&base_ty).into_var();
        let region = base_ty.clone_region();
        IdFamily {
            ty,
            region,
            base_ty: Some(base_ty),
        }
    }
    /// Get the pi type for a constructor family
    pub fn universal_pi(kind: &KindId) -> Pi {
        let universal_region = Region::with_unchecked(
            once(kind.clone_ty()).collect(),
            kind.clone_region(),
            kind.universe().clone_var(),
        );
        let base_ty = universal_region
            .param(0)
            .expect("Single, type parameter")
            .try_into_ty()
            .expect("Is type");
        let family_pi = Self::family_pi(&base_ty).into_ty();
        Pi::try_new(family_pi, universal_region).expect("Valid pi-type")
    }
    /// Get the pi type for an identity type family
    pub fn family_pi(base_ty: &TypeId) -> Pi {
        let region = Region::with_unchecked(
            [base_ty, base_ty].iter().copied().cloned().collect(),
            base_ty.clone_region(),
            base_ty.universe().clone_var(),
        );
        //TODO: proper target universe?
        Pi::try_new(base_ty.ty_kind().clone_ty(), region).expect("Valid pi-type")
    }
}

impl Typed for IdFamily {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
    #[inline]
    fn is_kind(&self) -> bool {
        false
    }
}

impl Regional for IdFamily {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.region.region()
    }
}

impl Apply for IdFamily {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        let base_ty_val = self.base_ty.as_ref().map(|ty| ty.as_val());
        match (args, base_ty_val) {
            ([], _) | ([_], Some(_)) => self.ty.apply_ty_in(args, ctx).map(Application::Symbolic),
            ([left, right], Some(base)) | ([base, left, right], None) => {
                if left.ty() != *base {
                    return Err(Error::TypeMismatch);
                }
                let id = Id::try_new(left.clone(), right.clone())?;
                Ok(Application::Success(&[], id.into_val()))
            }
            ([base, _], None) | ([base], None) => {
                let base_ty = Some(
                    base.clone()
                        .try_into_ty()
                        .map_err(|_| Error::NotATypeError)?,
                );
                let ty = self.ty.apply_ty_in(&args[..1], ctx)?;
                Ok(Application::Success(
                    &args[1..],
                    IdFamily {
                        base_ty,
                        region: ty.clone_region(),
                        ty: ty
                            .into_val()
                            .try_into()
                            .map_err(|_| Error::InvalidSubKind)?,
                    }
                    .into_val(),
                ))
            }
            _ => Err(Error::TooManyArgs),
        }
    }
}

impl Substitute for IdFamily {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<IdFamily, Error> {
        let base_ty = self
            .base_ty
            .as_ref()
            .map(|ty| ty.substitute_ty(ctx))
            .transpose()?;
        let ty: VarId<Pi> = self
            .ty
            .substitute(ctx)?
            .try_into()
            .map_err(|_| Error::InvalidSubKind)?;
        let region = if let Some(base_ty) = &base_ty {
            base_ty.gcr(&ty)?.clone_region()
        } else {
            ty.clone_region()
        };
        Ok(IdFamily {
            base_ty,
            ty,
            region,
        })
    }
}

substitute_to_valid!(IdFamily);

enum_convert! {
    impl InjectionRef<ValueEnum> for IdFamily {}
    impl TryFrom<NormalValue> for IdFamily { as ValueEnum, }
    impl TryFromRef<NormalValue> for IdFamily { as ValueEnum, }
}

impl From<IdFamily> for NormalValue {
    #[inline]
    fn from(id: IdFamily) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::IdFamily(id))
    }
}

impl Value for IdFamily {
    #[inline]
    fn no_deps(&self) -> usize {
        if self.base_ty.is_none() {
            0
        } else {
            1
        }
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        match ix {
            0 => self
                .base_ty
                .as_ref()
                .expect("Invalid zero-index into id family without base type")
                .as_val(),
            ix => panic!("Invalid index into id family's dependencies: {}", ix),
        }
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        self.into()
    }
}

/// A proof of identity for two values
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Id {
    /// The left value being compared
    left: ValId,
    /// The right value being compared
    right: ValId,
    /// The type of this identity type
    ty: KindId,
    /// The region of this identity type
    region: Region,
}

impl Id {
    /// Get the reflexivity type for a given value
    pub fn refl(value: ValId) -> Id {
        let ty = value.kind().id_kind();
        let region = value.clone_region();
        Id {
            left: value.clone(),
            right: value,
            ty,
            region,
        }
    }
    /// Get the identity type for comparison between two values of the same type
    pub fn try_new(left: ValId, right: ValId) -> Result<Id, Error> {
        if left.ty() != right.ty() {
            //TODO: subtyping
            return Err(Error::TypeMismatch);
        }
        let ty = left.kind().id_kind();
        let region = left.gcr(&right)?.clone_region();
        Ok(Id {
            left,
            right,
            region,
            ty, //TODO: this...
        })
    }
    /// Get the left of this identity type
    #[inline]
    pub fn left(&self) -> &ValId {
        &self.left
    }
    /// Get the right of this identity type
    #[inline]
    pub fn right(&self) -> &ValId {
        &self.right
    }
    /// Check whether this identity type is inhabited by `refl`, i.e. has judgementally equal left and right
    #[inline]
    pub fn has_refl(&self) -> bool {
        self.left == self.right
    }
}

impl Typed for Id {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
    #[inline]
    fn is_kind(&self) -> bool {
        false
    }
}

impl Regional for Id {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.region.region()
    }
}

impl Apply for Id {}

impl Substitute for Id {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Id, Error> {
        let left = self.left.substitute(ctx)?;
        let right = self.right.substitute(ctx)?;
        let ty: KindId = self
            .ty
            .substitute(ctx)?
            .try_into()
            .map_err(|_| Error::TypeMismatch)?;
        let region = left.gcr(&right)?.gcr(&ty)?.clone_region();
        Ok(Id {
            left,
            right,
            ty,
            region,
        })
    }
}

substitute_to_valid!(Id);

enum_convert! {
    impl InjectionRef<ValueEnum> for Id {}
    impl TryFrom<NormalValue> for Id { as ValueEnum, }
    impl TryFromRef<NormalValue> for Id { as ValueEnum, }
}

impl From<Id> for NormalValue {
    #[inline]
    fn from(id: Id) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Id(id))
    }
}

impl Value for Id {
    #[inline]
    fn no_deps(&self) -> usize {
        2
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        match ix {
            0 => &self.left,
            1 => &self.right,
            ix => panic!("Invalid index into an identity type's dependencies: {}", ix),
        }
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        self.into()
    }
}

impl Type for Id {
    #[inline]
    fn is_affine(&self) -> bool {
        //TODO
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        //TODO
        false
    }
}

/// The reflexivity axiom
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Refl {
    /// The base value
    value: ValId,
    /// The type of this invocation
    ty: TypeId,
    /// The region of this invocation
    region: Region,
}

impl Refl {
    /// Create a new instance of the reflexivity axiom on a given `ValId`
    #[inline]
    pub fn refl(value: ValId) -> Refl {
        let ty = Id::refl(value.clone()).into_ty();
        let region = value.clone_region();
        Refl { value, ty, region }
    }
}

impl Typed for Refl {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
    #[inline]
    fn is_kind(&self) -> bool {
        false
    }
}

impl Regional for Refl {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.region.region()
    }
}

impl Apply for Refl {}

impl Substitute for Refl {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Refl, Error> {
        let value = self.value.substitute(ctx)?;
        let ty = self.ty.substitute_ty(ctx)?;
        let region = value.lcr(&ty)?.clone_region();
        Ok(Refl { value, ty, region })
    }
}

substitute_to_valid!(Refl);

enum_convert! {
    impl InjectionRef<ValueEnum> for Refl {}
    impl TryFrom<NormalValue> for Refl { as ValueEnum, }
    impl TryFromRef<NormalValue> for Refl { as ValueEnum, }
}

impl From<Refl> for NormalValue {
    #[inline]
    fn from(refl: Refl) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Refl(refl))
    }
}

impl Value for Refl {
    #[inline]
    fn no_deps(&self) -> usize {
        1
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        match ix {
            0 => &self.value,
            ix => panic!("Invalid index into refl's dependencies: {}", ix),
        }
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        self.into()
    }
}

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

/// The non-dependent applicativity axiom
///
/// NOTE: applicativity of dependent functions is not yet supported, as we do not yet support transport along types.
///
/// We also do not yet have a family of non-dependent applicativity axioms, as we first need a supported way to pass a TyArr at all.
///
/// TODO: this should not be a primitive value, but rather a descriptor for a primitive value to be constructed
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Happly {
    /// The type of functions being applied
    ap_ty: VarId<Pi>,
    /// The particular function being applied, if any
    func: Option<ValId>,
    /// The type of this instance of the axiom
    ty: VarId<Pi>,
    /// The region of this axiom
    region: Region,
}

impl Happly {
    /// Create a new instance of the applicativity axiom for a pi type
    #[inline]
    pub fn try_new_pi(ap_ty: VarId<Pi>) -> Result<Happly, Error> {
        let ty = Self::pi_ty(ap_ty.clone())?;
        let region = ty.clone_region();
        Ok(Happly {
            ap_ty,
            func: None,
            region,
            ty: ty.into_var(),
        })
    }
    /// Create a new instance of the applicativity axiom for a given function
    #[inline]
    pub fn try_new_fn(ap_ty: VarId<Pi>, param_fn: ValId) -> Result<Happly, Error> {
        let ty = Self::fn_ty(&ap_ty, param_fn)?;
        let region = ty.clone_region();
        Ok(Happly {
            ap_ty,
            func: None,
            region,
            ty: ty.into_var(),
        })
    }
    /// Get the pi type corresponding to an instance of this axiom for a given function
    #[inline]
    pub fn fn_ty(ap_ty: &VarId<Pi>, param_fn: ValId) -> Result<Pi, Error> {
        if param_fn.ty() != *ap_ty {
            //TODO: subtyping?
            return Err(Error::TypeMismatch);
        }
        let domain = ap_ty.param_tys().clone();
        Self::fn_ty_helper(param_fn, domain)
    }
    /// Get the pi type corresponding to an instance of this axiom for a given function type
    #[inline]
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
        let mut identity_params = Vec::with_capacity(no_params);
        let mut left_params = Vec::with_capacity(no_params);
        let mut right_params = Vec::with_capacity(no_params);
        for (left, right) in left_region.params().zip(right_region.params()) {
            let left = left.into_val();
            let right = right.into_val();
            left_params.push(left.clone());
            right_params.push(right.clone());
            identity_params.push(Id::try_new(left, right)?.into_ty());
        }
        let identity_region = Region::with(identity_params.into(), right_region.clone_region())
            .expect("identity types lie in ap_ty's region");
        let left_ap = param_fn.applied(&left_params[..])?;
        let right_ap = param_fn.applied(&right_params[..])?;
        let result_id = Id::try_new(left_ap, right_ap)?;
        let arrow_pi =
            Pi::try_new(result_id.into_ty(), identity_region).expect("Arrow pi is valid");
        let right_pi = Pi::try_new(arrow_pi.into_ty(), right_region).expect("Right pi is valid");
        Ok(Pi::try_new(right_pi.into_ty(), left_region).expect("Left pi is valid"))
    }
}

impl Typed for Happly {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for IdFamily {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(identity family prettyprinting unimplemented)")
        }
    }

    impl PrettyPrint for Id {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(identity prettyprinting unimplemented)")
        }
    }

    impl PrettyPrint for Refl {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(refl prettyprinting unimplemented)")
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::logical::{binary_ty, Bool};
    use crate::typing::primitive::{Fin, Prop};
    use crate::value::Value;

    #[test]
    fn basic_bool_id() {
        let t = true.into_val();
        let f = false.into_val();
        let truthy = Id::refl(t.clone());
        assert!(truthy.has_refl());
        assert_eq!(*truthy.left(), t);
        assert_eq!(*truthy.right(), t);
        let falsey = Id::try_new(true.into_val(), false.into_val()).expect("Valid identity type");
        assert!(!falsey.has_refl());
        assert_ne!(truthy, falsey);
        assert_eq!(*falsey.left(), t);
        assert_eq!(*falsey.right(), f);

        let truthy = truthy.into_val();
        let falsey = falsey.into_val();
        assert_ne!(truthy, falsey);

        // Type/lifetime tests
        let prop = Prop.into_kind();
        assert_eq!(truthy.ty(), prop);
        assert_eq!(falsey.ty(), prop);
        assert_ne!(truthy, prop);
        assert_ne!(falsey, prop);
        assert_eq!(truthy.region(), Region::NULL);
        assert_eq!(falsey.region(), Region::NULL);

        // Refl true
        let refl_true = Refl::refl(true.into_val());
        assert_eq!(refl_true.ty(), truthy);
        assert_eq!(refl_true.region(), Region::NULL);

        // Typed full application
        let bool_family = IdFamily::family(Bool.into_ty());
        assert_eq!(
            bool_family.curried(&[t.clone(), t.clone()]).unwrap(),
            Application::Success(&[], truthy.clone())
        );
        assert_eq!(
            bool_family.curried(&[t.clone(), f.clone()]).unwrap(),
            Application::Success(&[], falsey.clone())
        );

        // Universal full application
        let base_family = IdFamily::universal(Fin.into_kind());
        assert_eq!(
            base_family
                .curried(&[Bool.into_val(), t.clone(), t.clone()])
                .unwrap(),
            Application::Success(&[], truthy)
        );
        assert_eq!(
            base_family.curried(&[Bool.into_val(), t, f]).unwrap(),
            Application::Success(&[], falsey)
        );

        // Universal type application
        //FIXME: partial type substitution
        /*
        assert_eq!(
            base_family.applied(&[Bool.into_val()]).unwrap(),
            bool_family.into_val()
        );
        */

        // Typed partial application
        //FIXME: partial application bug
        /*
        let partial_t = bool_family.applied(&[t.clone()]).expect("Valid partial application");
        let partial_bt = base_family.applied(&[Bool.into_val(), t.clone()]).expect("Valid partial application");
        */
    }

    #[test]
    fn happly_helpers() {
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
        let ap_type = Pi::try_new(left_pi.into_ty(), binary_region)
            .expect("Binary operation application type is valid")
            .into_ty();
        let ap_const = Happly::try_new_pi(binary_ty).unwrap();
        assert_eq!(ap_const.ty(), ap_type);
    }
}
