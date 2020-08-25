/*!
Lambda functions
*/
use super::pi::Pi;
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::region::{Parameter, Parametrized, Region, RegionBorrow, Regional};
use crate::typing::{Type, Typed};
use crate::value::{
    arr::{TySet, ValSet},
    Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueData, ValueEnum, VarId,
};
use crate::{debug_from_display, enum_convert, pretty_display, substitute_to_valid};
use std::convert::TryInto;

/// A lambda function
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Lambda {
    /// The result of this lambda function
    pub(crate) result: ValId,
    /// The type of this lambda function
    pub(crate) ty: VarId<Pi>,
    /// The direct dependencies of this lambda function
    pub(crate) deps: ValSet,
    /// The region of this lambda function
    pub(crate) def_region: Region,
}

impl Lambda {
    /// Create a new lambda function from a parametrized `ValId`
    pub fn new(result: Parametrized<ValId>) -> Lambda {
        let ty = VarId::from(Pi::ty(&result));
        let (def_region, result, deps) = result.destruct();
        Lambda {
            result,
            deps,
            def_region,
            ty,
        }
    }
    /// A utility constructor, which creates a new instance of the identity lambda for a given type
    pub fn id(ty: TypeId) -> Lambda {
        let tyset: TySet = std::iter::once(ty.clone()).collect();
        let def_region = Region::with_unchecked(
            tyset.as_arr().clone(),
            ty.clone_region(),
            ty.universe().clone_var(),
        );
        let result = Parameter::try_new(def_region.clone(), 0)
            .expect("Region has one parameter")
            .into();
        let ty: VarId<Pi> = Pi::try_new(ty, def_region.clone())
            .expect("Identity pi type is valid")
            .into();
        let deps = tyset.into_vals();
        Lambda {
            result,
            ty,
            deps,
            def_region,
        }
    }
    /// Attempt to create a new lambda function from a region and value
    pub fn try_new(value: ValId, region: Region) -> Result<Lambda, Error> {
        Ok(Self::new(Parametrized::try_new(value, region)?))
    }
    /// Get the defining region of this lambda function
    #[inline]
    pub fn def_region(&self) -> &Region {
        &self.def_region
    }
    /// Get the depth of the defining region of this lambda function
    #[inline]
    pub fn def_depth(&self) -> usize {
        let depth = self.depth() + 1;
        debug_assert_eq!(depth, self.def_region().depth());
        depth
    }
    /// Get the result of this lambda function
    #[inline]
    pub fn result(&self) -> &ValId {
        &self.result
    }
    /// Get the result type of this lambda function
    #[inline]
    pub fn result_ty(&self) -> &TypeId {
        &self.ty.result()
    }
    /// Get the type of this lambda function as a guaranteed pi type
    #[inline]
    pub fn get_ty(&self) -> &VarId<Pi> {
        &self.ty
    }
    /// Get the dependency-set of this lambda function
    #[inline]
    pub fn depset(&self) -> &ValSet {
        &self.deps
    }
}

impl Typed for Lambda {
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

impl Regional for Lambda {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.def_region.parent().borrow_region()
    }
}

impl Apply for Lambda {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        // Partial application check
        if self.def_region().len() > args.len() {
            return self
                .get_ty()
                .apply_ty_in(args, ctx)
                .map(Application::Symbolic);
        }

        // Initialize context
        if self.def_region().depth() > self.result().depth() {
            return Ok(Application::Success(
                &args[self.def_region().len()..],
                self.result().clone(),
            ));
        }

        let ctx = ctx.get_or_insert_with(EvalCtx::default);

        // Substitute
        let region = ctx.substitute_region(self.def_region(), args.iter().cloned(), false)?;

        // Evaluate the result
        let result = self.result().substitute(ctx);
        // Pop the evaluation context, and bubble up errors
        ctx.pop();
        let result = result?;
        let rest_args = &args[self.def_region().len().min(args.len())..];

        if let Some(region) = region {
            Lambda::try_new(result, region)
                .map(|lambda| Application::Success(rest_args, lambda.into()))
                .map_err(|_| Error::IncomparableRegions)
        } else {
            Ok(Application::Success(rest_args, result))
        }
    }
}

impl Value for Lambda {
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
        ValueEnum::Lambda(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
    #[inline]
    fn clone_depset(&self) -> ValSet {
        self.depset().clone()
    }
}

impl Substitute for Lambda {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Lambda, Error> {
        let result = self.result.substitute(ctx)?;
        let ty: VarId<Pi> = self
            .ty
            .substitute(ctx)?
            .try_into()
            .map_err(|_val| Error::InvalidSubKind)?;
        let deps: ValSet = self
            .deps
            .iter()
            .map(|d| d.substitute(ctx))
            .collect::<Result<_, _>>()?;
        let dep_gcr = Region::NULL.gcrs(deps.iter())?;
        let result_region = result.region();
        let def_region = if dep_gcr < result_region {
            result_region.clone_region()
        } else {
            Region::minimal_with(self.ty.param_tys().clone(), dep_gcr)?
        };
        Ok(Lambda {
            result,
            deps,
            ty,
            def_region,
        })
    }
}

impl ValueData for Lambda {}

substitute_to_valid!(Lambda);
debug_from_display!(Lambda);
pretty_display!(Lambda, "#lambda |...| {{...}}");
enum_convert! {
    impl InjectionRef<ValueEnum> for Lambda {}
    impl TryFrom<NormalValue> for Lambda { as ValueEnum, }
    impl TryFromRef<NormalValue> for Lambda { as ValueEnum, }
}

impl From<Lambda> for NormalValue {
    fn from(l: Lambda) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Lambda(l))
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Lambda {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            crate::region::prettyprint::prettyprint_parametrized(
                printer,
                fmt,
                &self.result,
                self.def_region(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{finite::Finite, logical::*};
    use crate::typing::Type;
    use crate::value::{expr::Sexpr, tuple::Tuple};

    #[test]
    fn boolean_identity_works_properly() {
        let id = Lambda::id(Bool.into()).into_val();
        for b in [true, false].iter().copied() {
            assert_eq!(
                Sexpr::try_new(vec![id.clone(), b.into()])
                    .expect("Valid application")
                    .into_val(),
                b.into_val()
            );
        }
    }

    #[test]
    fn small_finite_identity_works_properly() {
        let finite = Finite(16);
        let id = Lambda::id(finite.into()).into_val();
        for ix in finite.iter() {
            assert_eq!(
                Sexpr::try_new(vec![id.clone(), ix.clone().into()])
                    .expect("Valid application")
                    .into_val(),
                ix.into_val()
            );
        }
    }

    #[test]
    fn anchor_identity_works_properly() {
        let anchor = Tuple::const_anchor();
        let anchor_val = anchor.clone().into_val();
        let anchor_ty = anchor.ty().clone_ty();
        let id = Lambda::id(anchor_ty);
        let id_val = id.into_val();
        assert_eq!(
            Sexpr::try_new(vec![id_val, anchor_val.clone()])
                .unwrap()
                .into_val(),
            anchor_val
        );
    }

    #[test]
    fn boolean_mux_works_properly() {
        let region = Region::with(
            vec![Bool.into_ty(), Bool.into_ty(), Bool.into_ty()].into(),
            Region::NULL,
        )
        .unwrap();
        let select = region.param(0).unwrap().into_val();
        let high = region.param(1).unwrap().into_val();
        let low = region.param(2).unwrap().into_val();
        let not_select = Sexpr::try_new(vec![Not.into(), select.clone()])
            .unwrap()
            .into_val();
        let high_select = Sexpr::try_new(vec![And.into(), select, high])
            .unwrap()
            .into_val();
        let low_select = Sexpr::try_new(vec![And.into(), not_select, low])
            .unwrap()
            .into_val();
        let mux_res = Sexpr::try_new(vec![Or.into(), high_select, low_select])
            .unwrap()
            .into_val();
        let mux = Lambda::try_new(mux_res, region).unwrap().into_val();
        for h in [true, false].iter().copied() {
            let h = h.into_val();
            for l in [true, false].iter().copied() {
                let l = l.into_val();
                for s in [true, false].iter().copied() {
                    assert_eq!(
                        Sexpr::try_new(vec![mux.clone(), s.into(), h.clone(), l.clone()])
                            .unwrap()
                            .into_val(),
                        *if s { &h } else { &l }
                    )
                }
            }
        }
    }
}
