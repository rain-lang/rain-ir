/*!
Identity types and path induction
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

pub mod induction;

/// The identity type family, either of a given type or in general
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IdFamily {
    /// The type of this family
    ty: VarId<Pi>,
}

impl IdFamily {
    /// Get the constructor for all identity type families within a given kind
    pub fn universal(kind: KindId) -> IdFamily {
        let ty = Self::universal_pi(&kind).into_var();
        IdFamily { ty }
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
        self.ty.region()
    }
}

impl Apply for IdFamily {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        if args.len() > 3 {
            return Err(Error::TooManyArgs);
        }
        match args {
            [ty, left, right] => {
                if left.ty() != *ty {
                    Err(Error::TypeMismatch)
                } else {
                    Id::try_new(left.clone(), right.clone())
                        .map(|id| Application::Success(&[], id.into_val()))
                }
            }
            _ => self.ty.apply_ty_in(args, ctx).map(Application::Symbolic),
        }
    }
}

impl Substitute for IdFamily {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<IdFamily, Error> {
        let ty: VarId<Pi> = self
            .ty
            .substitute(ctx)?
            .try_into()
            .map_err(|_| Error::InvalidSubKind)?;
        Ok(IdFamily { ty })
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
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!("Invalid index into id family's dependencies: {}", ix)
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
            write!(fmt, "#id")
        }
    }

    impl PrettyPrint for Id {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(#id ")?;
            self.left.ty().prettyprint(printer, fmt)?;
            write!(fmt, " ")?;
            self.left.prettyprint(printer, fmt)?;
            write!(fmt, " ")?;
            self.right.prettyprint(printer, fmt)?;
            write!(fmt, ")")
        }
    }

    impl PrettyPrint for Refl {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(#refl ")?;
            self.value.ty().prettyprint(printer, fmt)?;
            write!(fmt, " ")?;
            self.value.prettyprint(printer, fmt)?;
            write!(fmt, ")")
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
        assert_eq!(truthy.region(), Region::NULL);
        assert_eq!(falsey.region(), Region::NULL);

        // Refl true
        let refl_true = Refl::refl(true.into_val());
        assert_eq!(refl_true.ty(), truthy);
        assert_eq!(refl_true.region(), Region::NULL);

        // Universal full application
        let base_family = IdFamily::universal(Fin.into_kind());
        assert_eq!(
            base_family
                .curried(&[Bool.into_val(), t.clone(), t.clone()])
                .unwrap(),
            Application::Success(&[], truthy)
        );
        assert_eq!(
            base_family
                .curried(&[Bool.into_val(), t.clone(), f])
                .unwrap(),
            Application::Success(&[], falsey)
        );

        // Universal type application

        let partial_bt = base_family
            .applied(&[Bool.into_val(), t.clone()])
            .expect("Valid partial application");
        //FIXME: should be equal to truthy, but sexpr currying is acting up...
        partial_bt.applied(&[t]).expect("Valid completion");
    }
}
