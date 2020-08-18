/*!
Pi types
*/
use crate::eval::{Apply, EvalCtx, Substitute};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::{Parameter, Parametrized, Region, RegionBorrow, Regional};
use crate::typing::{Type, Typed};
use crate::value::{
    arr::{TyArr, ValSet},
    Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueData, ValueEnum,
};
use crate::{debug_from_display, enum_convert, pretty_display, substitute_to_valid};
use std::convert::TryInto;

/// A pi type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Pi {
    /// The result type of this pi type
    result: TypeId,
    /// The direct dependencies of this pi type
    deps: ValSet,
    /// The lifetime of this pi type itself
    lt: Lifetime,
    /// The lifetime of this pi type's result
    result_lt: Lifetime,
}

impl Pi {
    /// Create a new pi type from a parametrized `TypeId` with a given result lifetime
    pub fn new(result: Parametrized<TypeId>, result_lt: &Lifetime) -> Result<Pi, Error> {
        let (region, result, deps, lt) = result.destruct();
        let result_lt = result_lt.in_region(Some(region))?;
        Ok(Pi {
            result,
            deps,
            lt,
            result_lt,
        })
    }
    /// Get the type associated with a parametrized `ValId`
    pub fn ty(param: &Parametrized<ValId>) -> Pi {
        Self::new(param.ty(), &*param.value().lifetime()).expect("Region conjunction should work!")
    }
    /// Attempt to create a new pi type from a region, type, and lifetime
    pub fn try_new(value: TypeId, region: Region, result_lt: &Lifetime) -> Result<Pi, Error> {
        Self::new(Parametrized::try_new(value, region)?, result_lt)
    }
    /// Get the result of this pi type
    #[inline]
    pub fn result(&self) -> &TypeId {
        &self.result
    }
    /// Get the result lifetime of this pi type
    #[inline]
    pub fn result_lt(&self) -> &Lifetime {
        &self.result_lt
    }
    /// Get the defining region of this pi type
    #[inline]
    pub fn def_region(&self) -> RegionBorrow {
        self.result_lt
            .region()
            .expect("Pi type cannot have a null region!")
    }
    /// Get the depth of the defining region of this pi type
    #[inline]
    pub fn def_depth(&self) -> usize {
        let depth = self.depth() + 1;
        debug_assert_eq!(depth, self.def_region().depth());
        depth
    }
    /// Get the parameter types of this pi type
    #[inline]
    pub fn param_tys(&self) -> &TyArr {
        self.def_region().data().param_tys()
    }
    /// Get the parameters of this pi type
    //TODO: parameter lifetimes...
    #[inline]
    pub fn params(&self) -> impl Iterator<Item = Parameter> + ExactSizeIterator {
        self.def_region().clone_region().params()
    }
}

impl Typed for Pi {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.def_region()
            .data()
            .universe()
            .borrow_var()
            .max(self.result().universe())
            .as_ty()
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

impl Live for Pi {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.result.lifetime()
    }
}

impl Regional for Pi {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.def_region()
            .parent()
            .map(|parent| parent.borrow_region())
    }
}

impl Apply for Pi {
    //TODO: allow limited pi-application?
}

impl Value for Pi {
    #[inline]
    fn no_deps(&self) -> usize {
        self.result.deps().len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self.result.deps()[ix]
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Pi(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for Pi {}

impl Type for Pi {
    #[inline]
    fn is_affine(&self) -> bool {
        //TODO: think about this...
        true
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        //TODO: think about this...
        true
    }
    #[inline]
    fn apply_ty_in(
        &self,
        args: &[ValId],
        lifetime: LifetimeBorrow,
        ctx: &mut Option<EvalCtx>,
    ) -> Result<(Lifetime, TypeId), Error> {
        // Rename context
        let ctx_handle = ctx;

        // Initialize context
        let ctx = ctx_handle.get_or_insert_with(|| EvalCtx::new(self.depth()));

        // Substitute
        let region =
            ctx.substitute_region(self.def_region().as_region(), args.iter().cloned(), false)?;

        // Evaluate the result type and lifetime
        let result = ctx.evaluate(self.result().as_val());
        let result_lt = ctx.evaluate_lt(&self.result_lt); //TODO: think about this...
                                                          // Pop the evaluation context
        ctx.pop();
        let result = result?;
        let result_lt = (result_lt? * lifetime)?;

        let rest_args = &args[self.def_region().len().min(args.len())..];

        if let Some(_region) = region {
            unimplemented!("Partial pi substitution")
        // let new_pi = Pi::try_new(
        //     result.try_into().expect("Partial pi result must be a type"),
        //     region,
        //     result_lt,
        // )?;
        //Ok((new_pi.lifetime().clone_lifetime(), new_pi.into()))
        } else if rest_args.is_empty() {
            Ok((
                result_lt,
                result.try_into().expect("Pi result must be a type"),
            ))
        } else {
            let result: TypeId = result.try_into().expect("Nested pi result must be a type");
            let (lt, ty) =
                result.apply_ty_in(rest_args, result_lt.borrow_lifetime(), ctx_handle)?;
            let lt = (lt + result_lt)?;
            Ok((lt, ty))
        }
    }
}

impl Substitute for Pi {
    fn substitute(&self, _ctx: &mut EvalCtx) -> Result<Pi, Error> {
        unimplemented!("Pi type substitution")
    }
}

impl From<Pi> for NormalValue {
    fn from(p: Pi) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Pi(p))
    }
}

substitute_to_valid!(Pi);
debug_from_display!(Pi);
pretty_display!(Pi, "#pi|...| {{...}}");
enum_convert! {
    impl InjectionRef<ValueEnum> for Pi {}
    impl TryFrom<NormalValue> for Pi { as ValueEnum, }
    impl TryFromRef<NormalValue> for Pi { as ValueEnum, }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Pi {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "#pi")?;
            crate::region::prettyprint::prettyprint_parametrized(
                printer,
                fmt,
                &self.result,
                self.def_region().as_region(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::logical::{binary_ty, unary_ty, BOOL_TY};

    #[test]
    fn basic_pi_application() {
        let unary = unary_ty();
        let binary = binary_ty();
        assert_eq!(
            unary
                .apply_ty(&[true.into()], LifetimeBorrow::STATIC)
                .unwrap(),
            (Lifetime::STATIC, (*BOOL_TY).clone_ty())
        );
        assert_eq!(
            binary
                .apply_ty(&[true.into(), false.into()], LifetimeBorrow::STATIC)
                .unwrap(),
            (Lifetime::STATIC, (*BOOL_TY).clone_ty())
        );
        //FIXME: this should return an error or succeed, but it's returning the wrong result now...
        // assert_eq!(
        //     binary.apply_ty(&[false.into()]).unwrap(),
        //     (Lifetime::STATIC, unary.clone_ty())
        // );
    }
}
