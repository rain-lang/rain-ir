/*!
Pi types
*/
use crate::eval::{Apply, EvalCtx, Substitute};
use crate::region::{Parameter, Parametrized, Region, RegionBorrow, Regional};
use crate::typing::{Type, Typed};
use crate::value::{
    arr::{TyArr, ValSet},
    Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueData, ValueEnum,
};
use crate::{debug_from_display, enum_convert, pretty_display, substitute_to_valid};

/// A pi type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Pi {
    /// The defining region of this pi type
    def_region: Region,
    /// The result type of this pi type
    result: TypeId,
    /// The direct dependencies of this pi type
    deps: ValSet,
}

impl Pi {
    /// Create a new pi type from a parametrized `TypeId` with a given result lifetime
    pub fn new(result: Parametrized<TypeId>) -> Result<Pi, Error> {
        let (def_region, result, deps) = result.destruct();
        Ok(Pi {
            result,
            deps,
            def_region,
        })
    }
    /// Get the type associated with a parametrized `ValId`
    pub fn ty(param: &Parametrized<ValId>) -> Pi {
        Self::new(param.ty()).expect("Region conjunction should work!")
    }
    /// Attempt to create a new pi type from a region, type, and lifetime
    pub fn try_new(value: TypeId, region: Region) -> Result<Pi, Error> {
        Self::new(Parametrized::try_new(value, region)?)
    }
    /// Get the result of this pi type
    #[inline]
    pub fn result(&self) -> &TypeId {
        &self.result
    }
    /// Get the defining region of this pi type
    #[inline]
    pub fn def_region(&self) -> &Region {
        &self.def_region
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
        self.def_region().param_tys()
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

impl Regional for Pi {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.def_region().parent().region()
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
    fn apply_ty_in(&self, args: &[ValId], ctx: &mut Option<EvalCtx>) -> Result<TypeId, Error> {
        // Empty argument case
        if args.is_empty() {
            return Ok(self.clone().into_ty());
        }

        // Skip null substitutions:
        let param_tys = self.param_tys();
        if args.len() >= param_tys.len()
            && (self.result.region() != self.def_region || args[..param_tys.len()] == **param_tys)
        {
            return if args.len() > param_tys.len() {
                self.result.apply_ty_in(&args[param_tys.len()..], ctx)
            } else {
                Ok(self.result.clone())
            };
        }

        // Rename context
        let ctx_handle = ctx;
        // Initialize context
        let ctx = ctx_handle.get_or_insert_with(|| EvalCtx::new(self.depth()));

        // Substitute
        let region = ctx.substitute_region(&self.def_region(), args.iter().cloned(), false)?;

        // Evaluate the result type and lifetime
        let result = self.result.substitute_ty(ctx);
        ctx.pop();
        let result = result?;

        let rest_args = &args[self.def_region().len().min(args.len())..];

        if let Some(def_region) = region {
            let pi = Pi::try_new(result, def_region)?;
            Ok(pi.into_ty())
        } else {
            result.apply_ty_in(rest_args, ctx_handle)
        }
    }
}

impl Substitute for Pi {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Pi, Error> {
        println!("STARTING PI SUBSTITUTION!");
        let result = self.result.substitute_ty(ctx)?;
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
            Region::minimal_with(self.param_tys().clone(), dep_gcr)?
        };
        println!(
            "PI SUBSTITUTION:\nSUBSTITUTING: {}\nRESULT = {}\nDEPS = {:#?}\n\n\n",
            self, result, deps,
        );
        Ok(Pi {
            result,
            deps,
            def_region,
        })
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
                self.def_region(),
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
        assert_eq!(unary.apply_ty(&[true.into()]).unwrap(), *BOOL_TY);
        assert_eq!(
            binary.apply_ty(&[true.into(), false.into()]).unwrap(),
            *BOOL_TY
        );
        //FIXME: this should return an error or succeed, but it's returning the wrong result now...
        // assert_eq!(
        //     binary.apply_ty(&[false.into()]).unwrap(),
        //     (Lifetime::STATIC, unary.clone_ty())
        // );
    }
}
