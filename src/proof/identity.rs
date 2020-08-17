/*!
Proofs of identity and equivalence.
*/
use crate::eval::Substitute;
use crate::eval::{Application, Apply, EvalCtx};
use crate::function::pi::Pi;
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::{Region, Regional};
use crate::typing::{Kind, Type, Typed};
use crate::value::{
    arr::TyArr, Error, KindId, NormalValue, TypeId, TypeRef, ValId, Value, ValueEnum, VarId,
};
use crate::{enum_convert, lifetime_region, substitute_to_valid};
use std::convert::TryInto;
//use either::Either;

/// The identity type family, either of a given type or in general
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IdFamily {
    /// The base type of this family, if any
    base_ty: Option<TypeId>,
    /// The type of this family
    ty: VarId<Pi>,
    /// The lifetime of this family
    lt: Lifetime,
}

impl IdFamily {
    /// Get the constructor for all identity type families within a given kind
    pub fn universal(kind: KindId) -> IdFamily {
        let ty = Self::universal_pi(&kind).into_var();
        let lt = kind.lifetime().clone_lifetime();
        IdFamily {
            ty,
            lt,
            base_ty: None,
        }
    }
    /// Get a given identity type family
    pub fn family(base_ty: TypeId) -> IdFamily {
        let ty = Self::family_pi(&base_ty).into_var();
        let lt = base_ty.lifetime().clone_lifetime();
        IdFamily {
            ty,
            lt,
            base_ty: Some(base_ty),
        }
    }
    /// Get the pi type for a constructor family
    pub fn universal_pi(kind: &KindId) -> Pi {
        let universal_region = Region::with_unchecked(
            std::iter::once(kind.clone_ty()).collect(),
            kind.cloned_region(),
            kind.universe().clone_var(),
        );
        let base_ty = universal_region
            .param(0)
            .expect("Single, type parameter")
            .try_into_ty()
            .expect("Is type");
        let family_pi = Self::family_pi(&base_ty).into_ty();
        let lt = family_pi.lifetime().clone_lifetime();
        Pi::try_new(family_pi, universal_region, &lt).expect("Valid pi-type")
    }
    /// Get the pi type for an identity type family
    pub fn family_pi(base_ty: &TypeId) -> Pi {
        let region = Region::with_unchecked(
            [base_ty, base_ty].iter().copied().cloned().collect(),
            base_ty.cloned_region(),
            base_ty.universe().clone_var(),
        );
        //TODO: proper target universe?
        Pi::try_new(base_ty.ty_kind().clone_ty(), region, &Lifetime::STATIC).expect("Valid pi-type")
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

impl Live for IdFamily {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.lt.borrow_lifetime()
    }
}

lifetime_region!(IdFamily);

impl Apply for IdFamily {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        let base_ty_val = self.base_ty.as_ref().map(|ty| ty.as_val());
        match (args, base_ty_val) {
            ([], _) | ([_], Some(_)) => {
                let (lt, ty) = self.ty.apply_ty_in(args, ctx)?;
                Ok(Application::Complete(lt, ty))
            }
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
                let (lt, ty) = self.ty.apply_ty_in(&args[..1], ctx)?;
                Ok(Application::Success(
                    &args[1..],
                    IdFamily {
                        base_ty,
                        lt,
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
        Ok(IdFamily {
            base_ty: self
                .base_ty
                .as_ref()
                .map(|ty| ty.substitute_ty(ctx))
                .transpose()?,
            ty: self
                .ty
                .substitute(ctx)?
                .try_into()
                .map_err(|_| Error::InvalidSubKind)?,
            lt: ctx.evaluate_lt(&self.lt)?,
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
    /// The lifetime of this identity type
    lt: Lifetime,
}

impl Id {
    /// Get the reflexivity type for a given value
    pub fn refl(value: ValId) -> Id {
        let lt = value.cloned_region().into();
        let ty = value.kind().id_kind();
        Id {
            left: value.clone(),
            right: value,
            ty,
            lt,
        }
    }
    /// Get the identity type for comparison between two values of the same type
    pub fn try_new(left: ValId, right: ValId) -> Result<Id, Error> {
        if left.ty() != right.ty() {
            //TODO: subtyping
            return Err(Error::TypeMismatch);
        }
        let ty = left.kind().id_kind();
        let lt = left.lcr(&right)?.cloned_region().into();
        Ok(Id {
            left,
            right,
            lt,
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

impl Live for Id {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.lt.borrow_lifetime()
    }
}

lifetime_region!(Id);

impl Apply for Id {}

impl Substitute for Id {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Id, Error> {
        Ok(Id {
            left: self.left.substitute(ctx)?,
            right: self.right.substitute(ctx)?,
            lt: ctx.evaluate_lt(&self.lt)?,
            ty: self
                .ty
                .substitute(ctx)?
                .try_into()
                .map_err(|_| Error::TypeMismatch)?,
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
    /// The lifetime of this invocation
    lt: Lifetime,
}

impl Refl {
    /// Create a new instance of the reflexivity axiom on a given `ValId`
    #[inline]
    pub fn refl(value: ValId) -> Refl {
        let ty = Id::refl(value.clone()).into_ty();
        let lt = value.cloned_region().into();
        Refl { value, ty, lt }
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

impl Live for Refl {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.lt.borrow_lifetime()
    }
}

lifetime_region!(Refl);

impl Apply for Refl {}

impl Substitute for Refl {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Refl, Error> {
        Ok(Refl {
            value: self.value.substitute(ctx)?,
            ty: self.ty.substitute_ty(ctx)?,
            lt: ctx.evaluate_lt(&self.lt)?,
        })
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

/// The non-dependent applicativity axiom
///
/// NOTE: applicativity of dependent functions is not yet supported, as we do not yet support transport along types.
///
/// We also do not yet have a family of non-dependent applicativity axioms, as we first need a supported way to pass a TyArr at all.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ApConst {
    /// The domain of functions being applied
    domain: TyArr,
    /// The target of functions being applied
    target: Option<TypeId>,
    /// The type of this instance of the axiom
    ty: VarId<Pi>,
    /// The lifetime of this instance of the axiom
    lt: Lifetime,
}

impl ApConst {
    /// Create a new instance of the applicativity axiom for a given domain and optional target
    #[inline]
    pub fn new(domain: TyArr, target: Option<TypeId>) -> Result<ApConst, Error> {
        let ty = Self::compute_ty(domain.clone(), target.clone())?;
        Ok(Self::new_helper(domain, target, ty))
    }
    /// Get the applicativity axiom over a given domain, with an unspecified target
    #[inline]
    pub fn over(domain: TyArr) -> Result<ApConst, Error> {
        let ty = Self::over_ty(domain.clone())?;
        Ok(Self::new_helper(domain, None, ty))
    }
    /// Get the applicativity axiom over a given domain with a given target
    #[inline]
    pub fn ap(domain: TyArr, target: TypeId) -> Result<ApConst, Error> {
        let ty = Self::ap_ty(domain.clone(), target.clone())?;
        Ok(Self::new_helper(domain, Some(target), ty))
    }
    /// Create an instance of the applicativity axiom with a given domain, optional target, and type
    fn new_helper(domain: TyArr, target: Option<TypeId>, ty: Pi) -> ApConst {
        let lt = ty.lifetime().clone_lifetime();
        ApConst {
            domain,
            target,
            ty: ty.into_var(),
            lt,
        }
    }
    /// Get the type of the applicativity axiom over a given domain, with an optional target
    #[inline]
    pub fn compute_ty(domain: TyArr, target: Option<TypeId>) -> Result<Pi, Error> {
        if let Some(target) = target {
            Self::ap_ty(domain, target)
        } else {
            Self::over_ty(domain)
        }
    }
    /// Get the type of the applicativity axiom over a given domain, with an unspecified target
    #[inline]
    pub fn over_ty(domain: TyArr) -> Result<Pi, Error> {
        unimplemented!("over_ty for domain {:?}", domain)
    }
    /// Get the type of the applicativity axiom over a given domain for a given target
    #[inline]
    pub fn ap_ty(domain: TyArr, target: TypeId) -> Result<Pi, Error> {
        unimplemented!("ap_ty for domain {:?}, target {}", domain, target)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::logical::Bool;
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
        assert_eq!(truthy.lifetime(), LifetimeBorrow::STATIC);
        assert_eq!(falsey.lifetime(), LifetimeBorrow::STATIC);

        // Refl true
        let refl_true = Refl::refl(true.into_val());
        assert_eq!(refl_true.ty(), truthy);
        assert_eq!(refl_true.lifetime(), LifetimeBorrow::STATIC);

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
}
