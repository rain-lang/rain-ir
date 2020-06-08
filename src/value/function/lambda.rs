/*!
Lambda functions
*/
use super::pi::Pi;
use crate::value::{
    eval::{Application, Apply, EvalCtx, Substitute},
    lifetime::Live,
    lifetime::{LifetimeBorrow, Parametrized, Region},
    typing::Typed,
    Error, TypeRef, ValId, Value, VarId,
};
use crate::{debug_from_display, pretty_display, substitute_to_valid};

/// A lambda function
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Lambda {
    /// The result of this lambda function
    result: Parametrized<ValId>,
    /// The type of this lambda function
    ty: VarId<Pi>,
}

impl Lambda {
    /// Create a new lambda function from a parametrized `ValId`
    pub fn new(result: Parametrized<ValId>) -> Lambda {
        let ty = VarId::from(Pi::ty(&result));
        Lambda { result, ty }
    }
    /// Attempt to create a new lambda function from a region and value
    pub fn try_new(value: ValId, region: Region) -> Result<Lambda, Error> {
        Ok(Self::new(Parametrized::try_new(value, region)?))
    }
    /// Get the defining region of this lambda function
    #[inline]
    pub fn def_region(&self) -> &Region {
        self.result.def_region()
    }
    /// Get the result of this lambda function
    #[inline]
    pub fn result(&self) -> &ValId {
        self.result.value()
    }
    /// Get the type of this lambda function as a guaranteed pi type
    #[inline]
    pub fn get_ty(&self) -> &VarId<Pi> {
        &self.ty
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
        self.result.lifetime()
    }
}

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
            let capacity = 0; //TODO
            let mut ctx = EvalCtx::with_capacity(capacity);
            return self.do_apply_in_ctx(args, inline, Some(&mut ctx));
        };
        if self.def_region().len() < args.len() && !inline {
            // Do a typecheck and lifetime check, then return application
            unimplemented!()
        }

        // Substitute
        let region = ctx.push_region(
            self.def_region(),
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
}

impl Substitute for Lambda {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Lambda, Error> {
        Ok(Lambda::new(self.result.substitute(ctx)?))
    }
}

substitute_to_valid!(Lambda);
debug_from_display!(Lambda);
pretty_display!(Lambda, "#lambda |...| {...}");

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
            self.result.prettyprint(printer, fmt)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::builder::Builder;
    use crate::prettyprinter::PrettyPrint;
    use crate::value::primitive::logical::{Bool, Not};

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
        assert_eq!(unary.deps().len(), 1);
        assert_eq!(unary.deps()[0], ValId::from(Bool));
        assert_eq!(id.ty(), unary);

        // Check type internally
        let (rest, jeq) = builder.parse_expr("#jeq[#typeof(id) unary]").unwrap();
        assert_eq!(rest, "");
        assert_eq!(jeq, ValId::from(true));

        // Check evaluations
        assert_eq!(builder.parse_expr("id #true"), Ok(("", ValId::from(true))));
        assert_eq!(
            builder.parse_expr("id #false"),
            Ok(("", ValId::from(false)))
        );

        // See if any stateful errors occur
        assert_eq!(
            builder.parse_expr("id #false"),
            Ok(("", ValId::from(false)))
        );
        assert_eq!(builder.parse_expr("id #true"), Ok(("", ValId::from(true))));
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
        assert_eq!(not.deps().len(), 1);
        assert_eq!(not.deps()[0], ValId::from(Not));
        let (rest, unary) = builder.parse_expr("unary").unwrap();
        assert_eq!(rest, "");
        assert_eq!(unary.deps().len(), 1);
        assert_eq!(unary.deps()[0], ValId::from(Bool));
        assert_eq!(not.ty(), unary);

        // Check depdendencies and types internally
        let (rest, jeq) = builder.parse_expr("#jeq[#typeof(not) unary]").unwrap();
        assert_eq!(rest, "");
        assert_eq!(jeq, ValId::from(true));

        // Check evaluations
        assert_eq!(
            builder.parse_expr("not #true"),
            Ok(("", ValId::from(false)))
        );
        assert_eq!(
            builder.parse_expr("not #false"),
            Ok(("", ValId::from(true)))
        );

        // See if any stateful errors occur
        assert_eq!(
            builder.parse_expr("not #false"),
            Ok(("", ValId::from(true)))
        );
        assert_eq!(
            builder.parse_expr("not #true"),
            Ok(("", ValId::from(false)))
        );
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
        assert_eq!(mux.deps().len(), 3); // and, or, not
        let (rest, ternary) = builder.parse_expr("ternary").unwrap();
        assert_eq!(rest, "");
        assert_eq!(ternary.deps().len(), 1);
        assert_eq!(ternary.deps()[0], ValId::from(Bool));
        assert_eq!(mux.ty(), ternary);

        // Check depdendencies and types internally
        let (rest, jeq) = builder.parse_expr("#jeq[#typeof(mux) ternary]").unwrap();
        assert_eq!(rest, "");
        assert_eq!(jeq, ValId::from(true));

        // Compute evaluations
        for (program, desired_result) in programs.iter() {
            let (rest, result) = builder.parse_expr(program).unwrap();
            assert_eq!(rest, "");
            assert_eq!(result, ValId::from(*desired_result));
        }
    }
}
