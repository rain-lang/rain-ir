/*!
Lambda functions
*/
use super::pi::Pi;
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::lifetime::Live;
use crate::lifetime::{Lifetime, LifetimeBorrow};
use crate::region::{Parameter, Parametrized, Region, RegionBorrow};
use crate::typing::Typed;
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
    result: ValId,
    /// The type of this lambda function
    ty: VarId<Pi>,
    /// The direct dependencies of this lambda function
    deps: ValSet,
    /// The lifetime of this lambda function
    lt: Lifetime,
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
        let region = Region::with(tyset.as_arr().clone(), Region::default());
        let result = Parameter::try_new(region.clone(), 0)
            .expect("Region has one parameter")
            .into();
        let ty: VarId<Pi> = Pi::try_new(ty, region.clone(), Lifetime::STATIC)
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
    /// Get the result of this lambda function
    #[inline]
    pub fn result(&self) -> &ValId {
        &self.result
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
    fn do_apply_in_ctx<'a>(
        &self,
        args: &'a [ValId],
        inline: bool,
        ctx: Option<&mut EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        let ctx = if let Some(ctx) = ctx {
            ctx
        } else {
            let eval_capacity = 0; //TODO
            let lt_capacity = 0; //TODO
            let mut ctx = EvalCtx::with_capacity(eval_capacity, lt_capacity);
            return self.do_apply_in_ctx(args, inline, Some(&mut ctx));
        };
        if self.def_region().len() > args.len() && !inline {
            // Do a typecheck and lifetime check, then return partial application
            unimplemented!(
                "Partial lambda application (args == {:?}, region_len == {:?})",
                args,
                self.def_region().len()
            )
        }

        // Substitute
        let region = ctx.push_region(
            self.def_region().as_region(),
            args.iter().cloned(),
            !ctx.is_checked(),
            inline,
        )?;

        // Evaluate the result
        let result = ctx.evaluate(self.result());
        // Pop the evaluation context
        ctx.pop();
        let result = result?;

        let rest_args = &args[self.def_region().len()..];

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
#[cfg(feature = "parser")]
mod tests {
    use super::*;
    use crate::builder::Builder;
    use crate::prettyprinter::PrettyPrint;
    // use crate::primitive::logical::Bool;

    #[test]
    fn bool_identity_lambda_works_properly() {
        let mut builder = Builder::<&str>::new();
        // Build the identity
        assert_eq!(builder.parse_statement("#let id = |x: #bool| x;"), Ok(""));
        // Build the unary type
        assert_eq!(
            builder.parse_statement("#let unary = #pi|_: #bool| #bool;"),
            Ok("")
        );

        // Check dependencies and type
        let (rest, id) = builder.parse_expr("id").unwrap();
        assert_eq!(rest, "");
        assert_eq!(id.deps().len(), 0);
        let (rest, unary) = builder.parse_expr("unary").unwrap();
        assert_eq!(rest, "");
        // FIXME: this
        // assert_eq!(unary.deps().len(), 1);
        // assert_eq!(unary.deps()[0], Bool.into_val());
        assert_eq!(id.ty(), unary);

        // Check type internally
        let (rest, jeq) = builder.parse_expr("#jeq[#typeof(id) unary]").unwrap();
        assert_eq!(rest, "");
        assert_eq!(jeq, true.into_val());

        // Check evaluations
        assert_eq!(builder.parse_expr("id #true"), Ok(("", true.into_val())));
        assert_eq!(builder.parse_expr("id #false"), Ok(("", false.into_val())));

        // See if any stateful errors occur
        assert_eq!(builder.parse_expr("id #false"), Ok(("", false.into_val())));
        assert_eq!(builder.parse_expr("id #true"), Ok(("", true.into_val())));
    }

    #[test]
    fn bool_negation_lambda_works_properly() {
        let mut builder = Builder::<&str>::new();
        // Build logical not as a lambda
        assert_eq!(
            builder.parse_statement("#let not = |x: #bool| (#not x);"),
            Ok("")
        );
        // Build the unary type
        assert_eq!(
            builder.parse_statement("#let unary = #pi|_: #bool| #bool;"),
            Ok("")
        );

        // Check dependencies and type externally
        let (rest, not) = builder.parse_expr("not").unwrap();
        assert_eq!(rest, "");
        let (rest, unary) = builder.parse_expr("unary").unwrap();
        assert_eq!(rest, "");
        assert_eq!(not.ty(), unary);

        // Check depdendencies and types internally
        let (rest, jeq) = builder.parse_expr("#jeq[#typeof(not) unary]").unwrap();
        assert_eq!(rest, "");
        assert_eq!(jeq, true.into_val());

        // Check evaluations
        assert_eq!(builder.parse_expr("not #true"), Ok(("", false.into_val())));
        assert_eq!(builder.parse_expr("not #false"), Ok(("", true.into_val())));

        // See if any stateful errors occur
        assert_eq!(builder.parse_expr("not #false"), Ok(("", true.into_val())));
        assert_eq!(builder.parse_expr("not #true"), Ok(("", false.into_val())));
    }

    #[test]
    fn mux_lambda_works_properly() {
        // Get mux evaluation programs
        let programs: Vec<_> = (0b000..=0b111)
            .map(|v| {
                let select = v & 0b100 != 0;
                let high = v & 0b010 != 0;
                let low = v & 0b001 != 0;
                (
                    format!("mux {} {} {}", select.prp(), high.prp(), low.prp()),
                    if select { high } else { low },
                )
            })
            .collect();

        let mut builder = Builder::<&str>::new();
        // Build mux as a lambda
        let mux_program =
            "#let mux = |select: #bool high: #bool low: #bool| (#or (#and select high) (#and (#not select) low));";
        assert_eq!(builder.parse_statement(mux_program), Ok(""));
        // Build the ternary type
        assert_eq!(
            builder.parse_statement("#let ternary = #pi|_: #bool _: #bool _: #bool| #bool;"),
            Ok("")
        );

        // Check dependencies and type externally
        let (rest, mux) = builder.parse_expr("mux").unwrap();
        assert_eq!(rest, "");
        // FIXME: this
        // assert_eq!(mux.deps().len(), 3); // and, or, not
        let (rest, ternary) = builder.parse_expr("ternary").unwrap();
        assert_eq!(rest, "");
        // FIXME: this
        // assert_eq!(ternary.deps().len(), 1);
        // assert_eq!(ternary.deps()[0], Bool.into_val());
        assert_eq!(mux.ty(), ternary);

        // Check depdendencies and types internally
        let (rest, jeq) = builder.parse_expr("#jeq[#typeof(mux) ternary]").unwrap();
        assert_eq!(rest, "");
        assert_eq!(jeq, true.into_val());

        // Compute evaluations
        for (program, desired_result) in programs.iter() {
            let (rest, result) = builder.parse_expr(program).unwrap();
            assert_eq!(rest, "");
            assert_eq!(result, desired_result.into_val());
        }
    }
}
