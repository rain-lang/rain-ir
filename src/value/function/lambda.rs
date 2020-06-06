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
    use crate::value::primitive::logical::Bool;

    #[test]
    fn identity_bool_lambda_works_properly() {
        let mut builder = Builder::<&str>::new();
        // Build the identity
        assert_eq!(builder.parse_statement("#let id = |x: #bool| x;"), Ok(""));
        // Build the unary type
        assert_eq!(builder.parse_statement("#let unary = #pi|_: #bool| #bool;"), Ok(""));

        // Check dependencies and type
        let (rest, id) = builder.parse_expr("id").unwrap();
        assert_eq!(rest, "");
        assert_eq!(id.deps().len(), 0);
        let (rest, unary) = builder.parse_expr("unary").unwrap();
        assert_eq!(rest, "");
        assert_eq!(unary.deps().len(), 1);
        assert_eq!(unary.deps()[0], ValId::from(Bool));
        assert_eq!(id.ty(), unary);
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
}
