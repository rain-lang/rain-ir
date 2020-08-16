/*!
Ternary operations and associated utilities

This module contains the [`Ternary`](Ternary) struct, `rain`'s answer to `if-then-else` expressions, and associated
utilities for working with ternary operations.
*/
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::function::{lambda::Lambda, pi::Pi};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::primitive::finite::Finite;
use crate::primitive::logical::BOOL_TY;
use crate::region::{Region, Regional};
use crate::typing::{Type, Typed};
use crate::value::{Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueEnum, VarId};
use crate::{lifetime_region, pretty_display, substitute_to_valid};
use std::convert::TryInto;

/// A ternary operation
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Ternary {
    /// The type of this ternary operation
    ty: VarId<Pi>,
    /// The lifetime of this ternary operation
    lt: Lifetime,
    /// The first branch of this ternary operation
    low: ValId,
    /// The second branch of this ternary operation
    high: ValId,
}

//debug_from_display!(Ternary);
pretty_display!(Ternary, "#ternary {...}");

impl Ternary {
    /// Construct a conditional node with the smallest possible type
    ///
    /// This constructs a conditional node, which is a function taking a single boolean parameter and returning
    /// `high` when the parameter is `true` and `low` when the parameter is `false`, assigned the smallest
    /// possible pi-type which can contain both `high` and `low`.
    ///
    /// # Example
    /// ```rust
    /// # use rain_ir::{control::ternary::Ternary, value::Value, primitive::finite::Finite};
    /// let high = Finite(8).ix(3).unwrap().into_val();
    /// let low = Finite(8).ix(2).unwrap().into_val();
    /// let conditional = Ternary::conditional(high.clone(), low.clone()).unwrap();
    /// assert_eq!(conditional.applied(&[true.into_val()]), Ok(high));
    /// assert_eq!(conditional.applied(&[false.into_val()]), Ok(low));
    /// ```
    #[inline]
    pub fn conditional(high: ValId, low: ValId) -> Result<Ternary, Error> {
        let high_ty = high.ty();
        let low_ty = low.ty();
        let lt = (low.lifetime() + high.lifetime())?;
        let unary_region = Region::with(
            std::iter::once(BOOL_TY.clone_ty()).collect(),
            lt.region().map(|region| region.clone_region()),
        )?;
        let ty = if high_ty == low_ty {
            Pi::try_new(high_ty.clone_ty(), unary_region, &lt)?.into()
        } else {
            unimplemented!("Dependently typed conditional: {} or {}", high, low);
        };
        Ok(Ternary { ty, lt, low, high })
    }
    /// Construct a switch ternary operation with the smallest possible type
    ///
    /// This constructs a ternary switch, which is a function taking in a single parameter of type `#finite(2)`
    /// and returning `high` when the parameter is equal to `#ix(2)[1]` and `low` when the parameter is equal to
    /// `#ix(2)[0]`, assigned the smallest possible pi-type which can contain both `high` and `low.
    ///
    /// This has the exact same behaviour as a `switch` node for `#finite(2)`, which in fact is normalized to this node type [^1]
    /// (when non-constant, as constant ternary nodes and switch nodes both normalize to a lambda)
    ///
    /// [^1]: `switch` nodes are not actually implemented yet, but their design is mostly completed
    ///
    /// # Example
    /// ```rust
    /// # use rain_ir::{control::ternary::Ternary, value::Value, primitive::finite::Finite};
    /// let high = Finite(8).ix(3).unwrap().into_val();
    /// let low = Finite(8).ix(2).unwrap().into_val();
    /// let switch = Ternary::switch(high.clone(), low.clone()).unwrap();
    /// let one = Finite(2).ix(1).unwrap().into_val();
    /// let zero = Finite(2).ix(0).unwrap().into_val();
    /// assert_eq!(switch.applied(&[one]), Ok(high));
    /// assert_eq!(switch.applied(&[zero]), Ok(low));
    /// ```
    #[inline]
    pub fn switch(high: ValId, low: ValId) -> Result<Ternary, Error> {
        let high_ty = high.ty();
        let low_ty = low.ty();
        let lt = (low.lifetime() + high.lifetime())?;
        let switch_region = Region::with(
            std::iter::once(Finite(2).into_ty()).collect(),
            lt.region().map(|region| region.clone_region()),
        )?;
        let ty = if high_ty == low_ty {
            Pi::try_new(high_ty.clone_ty(), switch_region, &lt)?.into()
        } else {
            unimplemented!("Dependently typed conditional: {} or {}", high, low);
        };
        Ok(Ternary { ty, lt, low, high })
    }
    /// Get the parameter type type of this ternary operation
    ///
    /// Ternary operations always consume a single parameter, which currently can either be of type `#bool` or `#finite(2)`.
    /// This returns the type of that parameter.
    ///
    /// # Example
    /// ```rust
    /// # use rain_ir::{control::ternary::Ternary, value::Value, primitive::{finite::Finite, logical::Bool}};
    /// let conditional = Ternary::conditional(true.into_val(), false.into_val()).unwrap();
    /// let switch = Ternary::switch(true.into_val(), false.into_val()).unwrap();
    /// assert_eq!(*conditional.param_ty(), Bool.into_val());
    /// assert_eq!(*switch.param_ty(), Finite(2).into_val());
    /// ```
    #[inline]
    pub fn param_ty(&self) -> &TypeId {
        &self.ty.param_tys()[0]
    }
    /// Get the type of this ternary operation
    ///
    /// This is provided as a convenience method as the type of a ternary operation is guaranteed to be a valid pi-type, so
    /// the need for a downcast is avoided.
    #[inline]
    pub fn get_ty(&self) -> &VarId<Pi> {
        &self.ty
    }
    /// Get the first branch of this ternary operation
    #[inline]
    pub fn low(&self) -> &ValId {
        &self.low
    }
    /// Get the second branch of this ternary operation
    #[inline]
    pub fn high(&self) -> &ValId {
        &self.high
    }
    /// Get whether this ternary node is constant. Should always be `false` for a normalized node!
    ///
    /// Note that constant ternary nodes are normalized into constant lambda functions.
    #[inline]
    pub fn is_const(&self) -> bool {
        self.low == self.high
    }
    /// Get the ternary kind of this node
    #[inline]
    pub fn ternary_kind(&self) -> TernaryKind {
        match self.param_ty().as_enum() {
            ValueEnum::BoolTy(_) => TernaryKind::Bool,
            ValueEnum::Finite(f) if *f == Finite(2) => TernaryKind::Switch,
            p => panic!("Invalid ternary parameter type {}", p),
        }
    }
}
/// Kinds of ternary node
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TernaryKind {
    /// A boolean branch
    Bool,
    /// A branch on `#finite(2)`
    Switch,
}

impl Typed for Ternary {
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_kind(&self) -> bool {
        false
    }
}

impl Live for Ternary {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.lt.borrow_lifetime()
    }
}

lifetime_region!(Ternary);

impl Apply for Ternary {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        // Empty application
        if args.is_empty() {
            return Ok(Application::Complete(self.lt.clone(), self.ty().clone_ty()));
        }
        match self.ternary_kind() {
            TernaryKind::Bool => {
                if let ValueEnum::Bool(b) = args[0].as_enum() {
                    let rest = &args[1..];
                    if *b {
                        Ok(Application::Success(rest, self.high.clone()))
                    } else {
                        Ok(Application::Success(rest, self.low.clone()))
                    }
                } else {
                    self.ty
                        .apply_ty_in(args, ctx)
                        .map(|(lt, ty)| Application::Stop(lt, ty))
                }
            }
            TernaryKind::Switch => {
                if let ValueEnum::Index(ix) = args[0].as_enum() {
                    if *ix.get_ty() != Finite(2) {
                        return Err(Error::TypeMismatch);
                    }
                    let rest = &args[1..];
                    if ix.ix() != 0 {
                        Ok(Application::Success(rest, self.high.clone()))
                    } else {
                        Ok(Application::Success(rest, self.low.clone()))
                    }
                } else {
                    self.ty
                        .apply_ty_in(args, ctx)
                        .map(|(lt, ty)| Application::Stop(lt, ty))
                }
            }
        }
    }
}

impl Substitute for Ternary {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Ternary, Error> {
        Ok(Ternary {
            ty: self
                .ty
                .substitute(ctx)?
                .try_into()
                //TODO
                .map_err(|_val| Error::InvalidSubKind)?,
            lt: ctx.evaluate_lt(&self.lt)?,
            low: self.low.substitute(ctx)?,
            high: self.high.substitute(ctx)?,
        })
    }
}

substitute_to_valid!(Ternary);

impl Value for Ternary {
    #[inline]
    fn no_deps(&self) -> usize {
        2
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        match ix {
            0 => &self.low,
            1 => &self.high,
            _ => panic!("Invalid dependency index {}", ix),
        }
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl From<Ternary> for ValueEnum {
    fn from(ternary: Ternary) -> ValueEnum {
        ValueEnum::Ternary(ternary)
    }
}

impl From<Ternary> for NormalValue {
    fn from(ternary: Ternary) -> NormalValue {
        if ternary.is_const() {
            // Cast this ternary to a constant lambda
            NormalValue(ValueEnum::Lambda(Lambda {
                result: ternary.high,
                ty: ternary.ty,
                lt: ternary.lt,
                deps: std::iter::once(ternary.low).collect(),
            }))
        } else {
            NormalValue(ValueEnum::Ternary(ternary))
        }
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Ternary {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(ternary prettyprinting unimplemented)")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::logical::unary_region;
    use crate::value::expr::Sexpr;

    #[test]
    fn basic_conditional_application() {
        let finite2: VarId<Finite> = Finite(2).into();
        let finite: VarId<Finite> = Finite(6).into();
        let high = finite.clone().ix(3).unwrap().into_val();
        let low = finite.ix(1).unwrap().into_val();
        let ternary = Ternary::conditional(high.clone(), low.clone()).unwrap();
        let ix1 = finite2.clone().ix(1).unwrap().into_val();
        let ix0 = finite2.ix(0).unwrap().into_val();
        assert_eq!(
            ternary.apply(&[true.into()]).unwrap(),
            Application::Success(&[], high.clone())
        );
        assert_eq!(
            ternary.apply(&[false.into()]).unwrap(),
            Application::Success(&[], low.clone())
        );
        assert!(ternary.apply(&[ix0.clone()]).is_err());
        let ternary = ternary.into_val();
        assert_eq!(
            ternary.apply(&[true.into()]).unwrap(),
            Application::Success(&[], high.clone())
        );
        assert_eq!(
            ternary.apply(&[false.into()]).unwrap(),
            Application::Success(&[], low.clone())
        );
        assert!(ternary.apply(&[ix0]).is_err());
        assert_eq!(
            Sexpr::try_new(vec![ternary.clone(), true.into()])
                .unwrap()
                .into_val(),
            high
        );
        assert_eq!(
            Sexpr::try_new(vec![ternary.clone(), false.into()])
                .unwrap()
                .into_val(),
            low
        );
        assert!(Sexpr::try_new(vec![ternary, ix1]).is_err());
    }

    #[test]
    fn basic_switch_application() {
        let finite2: VarId<Finite> = Finite(2).into();
        let finite: VarId<Finite> = Finite(9).into();
        let high = finite.clone().ix(4).unwrap().into_val();
        let low = finite.ix(7).unwrap().into_val();
        let ternary = Ternary::switch(high.clone(), low.clone()).unwrap();
        let ix1 = finite2.clone().ix(1).unwrap().into_val();
        let ix0 = finite2.ix(0).unwrap().into_val();
        assert_eq!(
            ternary.apply(&[ix1.clone()]).unwrap(),
            Application::Success(&[], high.clone())
        );
        assert_eq!(
            ternary.apply(&[ix0.clone()]).unwrap(),
            Application::Success(&[], low.clone())
        );
        assert!(ternary.apply(&[true.into()]).is_err());
        let ternary = ternary.into_val();
        assert_eq!(
            ternary.apply(&[ix1.clone()]).unwrap(),
            Application::Success(&[], high.clone())
        );
        assert_eq!(
            ternary.apply(&[ix0.clone()]).unwrap(),
            Application::Success(&[], low.clone())
        );
        assert!(ternary.apply(&[true.into()]).is_err());
        assert_eq!(
            Sexpr::try_new(vec![ternary.clone(), ix1])
                .unwrap()
                .into_val(),
            high
        );
        assert_eq!(
            Sexpr::try_new(vec![ternary.clone(), ix0])
                .unwrap()
                .into_val(),
            low
        );
        assert!(Sexpr::try_new(vec![ternary, true.into()]).is_err());
    }

    #[test]
    fn constant_conditional_application_and_norm() {
        //FIXME: lambda type mismatches
        // let finite2: VarId<Finite> = Finite(2).into();
        let finite: VarId<Finite> = Finite(6).into();
        let ix = finite.ix(3).unwrap().into_val();
        let ternary = Ternary::conditional(ix.clone(), ix.clone()).unwrap();
        let const_lambda = Lambda::try_new(ix.clone(), unary_region()).unwrap();
        // let ix1 = finite2.clone().ix(1).unwrap().into_val();
        // let ix0 = finite2.clone().ix(0).unwrap().into_val();
        assert_eq!(
            ternary.apply(&[true.into()]).unwrap(),
            Application::Success(&[], ix.clone())
        );
        assert_eq!(
            ternary.apply(&[false.into()]).unwrap(),
            Application::Success(&[], ix.clone())
        );
        assert_eq!(
            ternary.clone().into_norm(),
            const_lambda.clone().into_norm()
        );
        // assert!(ternary.apply(&[ix1.clone()]).is_err());
        let ternary = ternary.into_val();
        assert_eq!(ternary, const_lambda.into_val());
        assert_eq!(
            ternary.apply(&[true.into()]).unwrap(),
            Application::Success(&[], ix.clone())
        );
        assert_eq!(
            ternary.apply(&[false.into()]).unwrap(),
            Application::Success(&[], ix.clone())
        );
        // assert!(ternary.apply(&[ix1]).is_err());
        assert_eq!(
            Sexpr::try_new(vec![ternary.clone(), true.into()])
                .unwrap()
                .into_val(),
            ix
        );
        assert_eq!(
            Sexpr::try_new(vec![ternary, false.into()])
                .unwrap()
                .into_val(),
            ix
        );
        // assert!(Sexpr::try_new(vec![ternary.clone(), ix0]).is_err());
    }

    #[test]
    fn constant_switch_application_and_norm() {
        let finite2: VarId<Finite> = Finite(2).into();
        let finite: VarId<Finite> = Finite(6).into();
        let ix = finite.ix(3).unwrap().into_val();
        let ternary = Ternary::switch(ix.clone(), ix.clone()).unwrap();
        let ix1 = finite2.clone().ix(1).unwrap().into_val();
        let ix0 = finite2.clone().ix(0).unwrap().into_val();
        let finite_region = Region::with(std::iter::once(finite2.into_ty()).collect(), None).unwrap();
        let const_lambda = Lambda::try_new(ix.clone(), finite_region).unwrap();
        assert_eq!(
            ternary.apply(&[ix1.clone()]).unwrap(),
            Application::Success(&[], ix.clone())
        );
        assert_eq!(
            ternary.apply(&[ix0.clone()]).unwrap(),
            Application::Success(&[], ix.clone())
        );
        assert_eq!(
            ternary.clone().into_norm(),
            const_lambda.clone().into_norm()
        );
        let ternary = ternary.into_val();
        assert_eq!(ternary, const_lambda.into_val());
        assert_eq!(
            ternary.apply(&[ix1.clone()]).unwrap(),
            Application::Success(&[], ix.clone())
        );
        assert_eq!(
            ternary.apply(&[ix0.clone()]).unwrap(),
            Application::Success(&[], ix.clone())
        );
        assert_eq!(
            Sexpr::try_new(vec![ternary.clone(), ix1])
                .unwrap()
                .into_val(),
            ix
        );
        assert_eq!(Sexpr::try_new(vec![ternary, ix0]).unwrap().into_val(), ix);
    }

    #[test]
    fn nested_ternary_xor() {
        let id = Ternary::conditional(true.into(), false.into()).unwrap();
        assert_eq!(id.lifetime(), Lifetime::STATIC);
        let not = Ternary::conditional(false.into(), true.into()).unwrap();
        assert_eq!(not.lifetime(), Lifetime::STATIC);
        let xor = Ternary::conditional(not.into(), id.into()).unwrap();
        assert_eq!(xor.lifetime(), Lifetime::STATIC);
        let xor = xor.into_val();
        assert_eq!(xor.lifetime(), Lifetime::STATIC);
        for l in [true, false].iter().copied() {
            let lv = l.into_val();
            for r in [true, false].iter().copied() {
                let x = l != r;
                let rv = r.into_val();
                assert_eq!(
                    xor.clone().applied(&[lv.clone(), rv.clone()]),
                    Ok(x.into_val())
                );
            }
        }
    }
}