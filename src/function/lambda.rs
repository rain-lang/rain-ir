/*!
Lambda functions
*/
use super::pi::Pi;
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::lifetime::Live;
use crate::lifetime::{Lifetime, LifetimeBorrow};
use crate::region::{Parameter, Parametrized, Region, RegionBorrow, Regional};
use crate::typing::{Type, Typed};
use crate::value::{
    arr::{TySet, ValSet},
    Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueData, ValueEnum, VarId,
};
use crate::{
    debug_from_display, enum_convert, lifetime_region, pretty_display, substitute_to_valid,
};
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
    /// The lifetime of this lambda function
    pub(crate) lt: Lifetime,
}

impl Lambda {
    /// Create a new lambda function from a parametrized `ValId`
    pub fn new(result: Parametrized<ValId>) -> Lambda {
        let ty = VarId::from(Pi::ty(&result));
        let (_region, result, deps, lt) = result.destruct();
        Lambda {
            result,
            deps,
            lt,
            ty,
        }
    }
    /// A utility constructor, which creates a new instance of the identity lambda for a given type
    pub fn id(ty: TypeId) -> Lambda {
        let tyset: TySet = std::iter::once(ty.clone()).collect();
        let region = Region::with(tyset.as_arr().clone(), None);
        let result = Parameter::try_new(region.clone(), 0)
            .expect("Region has one parameter")
            .into();
        let ty: VarId<Pi> = Pi::try_new(ty, region, Lifetime::STATIC)
            .expect("Identity pi type is valid")
            .into();
        let lt = ty.lifetime().clone_lifetime(); //TODO: someday...
        let deps = tyset.into_vals();
        Lambda {
            result,
            ty,
            deps,
            lt,
        }
    }
    /// Attempt to create a new lambda function from a region and value
    pub fn try_new(value: ValId, region: Region) -> Result<Lambda, Error> {
        Ok(Self::new(Parametrized::try_new(value, region)?))
    }
    /// Get the defining region of this lambda function
    #[inline]
    pub fn def_region(&self) -> RegionBorrow {
        self.ty.def_region()
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
    /// Get the result lifetime of this lambda function
    #[inline]
    pub fn result_lt(&self) -> &Lifetime {
        &self.ty.result_lt()
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
}

impl Live for Lambda {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.lt.borrow_lifetime()
    }
}

lifetime_region!(Lambda);

impl Apply for Lambda {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        // Partial application check
        if self.def_region().len() > args.len() {
            let (lt, ty) = self.get_ty().apply_ty_in(args, ctx)?;
            return Ok(Application::Incomplete(lt, ty));
        }

        // Initialize context
        if self.def_region().depth() > self.result().depth() {
            return Ok(Application::Success(
                &args[self.def_region().len()..],
                self.result().clone(),
            ));
        }

        let ctx = ctx.get_or_insert_with(|| EvalCtx::new(self.depth()));

        // Substitute
        let region =
            ctx.substitute_region(self.def_region().as_region(), args.iter().cloned(), false)?;

        // Evaluate the result's lifetime
        let result_lt = ctx.evaluate_lt(self.result_lt());
        // Evaluate the result if it has a valid lifetime
        let result = if result_lt.is_ok() {
            ctx.evaluate(self.result())
        } else {
            Err(Error::LifetimeError)
        };
        // Pop the evaluation context, and bubble up errors
        ctx.pop();
        let _result_lt = result_lt?;
        let result = result?;

        // Cast the result into it's lifetime
        //TODO: casting

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
        Ok(Lambda {
            result: self.result.substitute(ctx)?,
            ty: self
                .ty
                .substitute(ctx)?
                .try_into()
                //TODO
                .map_err(|_val| Error::InvalidSubKind)?,
            deps: self
                .deps
                .iter()
                .map(|d| d.substitute(ctx))
                .collect::<Result<_, _>>()?,
            lt: ctx.evaluate_lt(&self.lt)?,
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
        NormalValue(ValueEnum::Lambda(l))
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
                self.def_region().as_region(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifetime::Color;
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
        let id_val = id.clone().into_val();
        assert_eq!(
            Sexpr::try_new(vec![id_val, anchor_val.clone()])
                .unwrap()
                .into_val(),
            anchor_val
        );
        // Different lifetimes!
        assert_ne!(*id.result(), anchor_val);
        // Specifically, color static vs. color param 0
        let region = id.def_region();
        let color = Color::param(region.as_region(), 0).unwrap();
        let lt = Lifetime::owns(color);
        assert_eq!(id.result().lifetime(), lt);
        //TODO: fix this
        //assert_eq!(*id.result().lifetime(), *id.result_lt());
        assert_eq!(anchor.lifetime(), Lifetime::STATIC);
    }

    #[test]
    fn boolean_mux_works_properly() {
        let region = Region::with(
            vec![Bool.into_ty(), Bool.into_ty(), Bool.into_ty()].into(),
            None,
        );
        let select = region.clone().param(0).unwrap().into_val();
        let high = region.clone().param(1).unwrap().into_val();
        let low = region.clone().param(2).unwrap().into_val();
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
