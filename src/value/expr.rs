/*!
`rain` expressions
*/
use super::{arr::ValArr, Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueData, ValueEnum};
use crate::enum_convert;
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::lifetime::{LifetimeBorrow, Live};
use crate::primitive::UNIT_TY;
use crate::region::{Region, Regional};
use crate::typing::Typed;
use crate::{debug_from_display, pretty_display, substitute_to_valid, valarr};
use std::ops::Deref;

/// An S-expression
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Sexpr {
    /// The arguments of this S-expression
    pub(super) args: ValArr,
    /// The (cached) region of this S-expression
    pub(super) region: Region,
    /// The (cached) type of this S-expression
    pub(super) ty: TypeId,
}

debug_from_display!(Sexpr);
pretty_display!(Sexpr, "(...)");

enum_convert! {
    impl InjectionRef<ValueEnum> for Sexpr {}
    impl TryFrom<NormalValue> for Sexpr { as ValueEnum, }
    impl TryFromRef<NormalValue> for Sexpr { as ValueEnum, }
}

impl Sexpr {
    /// Create a new S-expression fron unchecked components
    pub(crate) fn new_unchecked(args: ValArr, region: Region, ty: TypeId) -> Sexpr {
        Sexpr { args, region, ty }
    }
    /// Attempt to create an S-expression from an owned argument list, evaluating as necessary.
    pub fn try_new(mut args: Vec<ValId>) -> Result<Sexpr, Error> {
        // Simple cases
        match args.len() {
            0 => return Ok(Self::unit()),
            1 => return Ok(Self::singleton(args.swap_remove(0))),
            _ => {}
        }
        // Expand sexprs in the first argument
        if let ValueEnum::Sexpr(s) = args[0].as_enum() {
            if s.is_empty() {
                return Err(Error::EmptySexprApp); // Special error for unit application
            }
            let mut new_args = Vec::with_capacity(args.len() + s.len());
            new_args.extend(s.iter().cloned());
            new_args.extend(args.drain(1..));
            args = new_args;
        }
        // General case
        let ty = match args[0].apply(&args[1..])? {
            Application::Success(rest, valid) => return Self::applied_with(valid, rest),
            Application::Symbolic(ty) => ty,
        };
        let region = ty.gcrs(args.iter())?.clone_region();
        Ok(Sexpr {
            args: args.into(),
            region,
            ty,
        })
    }
    /// Attempt to create an S-expression from an un-owned argument-list, evaluating as necessary
    pub fn eval(args: &[ValId]) -> Result<Sexpr, Error> {
        match args.len() {
            0 => Ok(Self::unit()),
            _ => Self::applied_with(args[0].clone(), &args[..]),
        }
    }
    /// Attempt to create an S-expression by applying an argument to an argument list, evaluating as necessary.
    pub fn applied_with(mut f: ValId, mut args: &[ValId]) -> Result<Sexpr, Error> {
        while !args.is_empty() {
            match f.apply(args)? {
                Application::Success(rest, v) => {
                    args = rest;
                    f = v;
                    continue;
                }
                Application::Symbolic(ty) => {
                    let mut a = Vec::with_capacity(1 + args.len());
                    a.push(f);
                    a.clone_from_slice(args);
                    let region = ty.gcrs(args.iter())?.clone_region();
                    return Ok(Sexpr {
                        args: a.into(),
                        ty,
                        region,
                    });
                }
            };
        }
        Ok(Self::singleton(f))
    }
    /// Create an S-expression corresponding to the unit value
    pub fn unit() -> Sexpr {
        Sexpr {
            args: ValArr::EMPTY,
            region: Region::NULL,
            ty: UNIT_TY.clone_as_ty(),
        }
    }
    /// Create an S-expression corresponding to a singleton value
    pub fn singleton(value: ValId) -> Sexpr {
        if let ValueEnum::Sexpr(s) = value.as_enum() {
            // Edge case
            return s.clone();
        }
        let ty = value.clone_ty();
        let region = value.clone_region();
        Sexpr {
            args: valarr![value],
            region,
            ty,
        }
    }
}

impl ValueData for Sexpr {}

impl Live for Sexpr {
    fn lifetime(&self) -> LifetimeBorrow {
        //FIXME: this
        self.region.region().into()
    }
}

impl Typed for Sexpr {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        match self.len() {
            0 => false,
            1 => self[0].is_ty(),
            //TODO: this
            _ => self.ty().is_kind(),
        }
    }
    #[inline]
    fn is_kind(&self) -> bool {
        match self.len() {
            0 => false,
            1 => self[0].is_kind(),
            //TODO: this
            _ => self.ty().ty().is_kind(),
        }
    }
}

impl Value for Sexpr {
    #[inline]
    fn no_deps(&self) -> usize {
        self.len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self[ix]
    }
    #[inline]
    fn dep_owned(&self, _ix: usize) -> bool {
        //TODO: generalize once pi nodes are fixed
        true
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Sexpr(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl Deref for Sexpr {
    type Target = ValArr;
    fn deref(&self) -> &ValArr {
        &self.args
    }
}

impl Apply for Sexpr {}

impl Substitute for Sexpr {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Sexpr, Error> {
        let args: Result<_, _> = self
            .args
            .iter()
            .cloned()
            .map(|val| val.substitute(ctx))
            .collect();
        //TODO: this
        Sexpr::try_new(args?)
    }
}

impl From<Sexpr> for NormalValue {
    fn from(sexpr: Sexpr) -> NormalValue {
        if sexpr.len() == 0 {
            return ().into();
        }
        if sexpr.len() == 1 && sexpr[0].ty() == sexpr.ty {
            return sexpr[0].as_norm().clone();
        }
        NormalValue::assert_normal(ValueEnum::Sexpr(sexpr))
    }
}

substitute_to_valid!(Sexpr);

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use crate::tokens::*;
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Sexpr {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "{}", SEXPR_OPEN)?;
            let mut first = true;
            for value in self.iter() {
                if !first {
                    write!(fmt, " ")?;
                }
                first = false;
                value.prettyprint(printer, fmt)?;
            }
            write!(fmt, "{}", SEXPR_CLOSE)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    /// Test converting the unit S-expression to and from ValueEnum/NormalValue works properly
    #[test]
    fn unit_value_construction() {
        let unit_sexpr = Sexpr::unit();
        let unit_value = ValueEnum::Sexpr(unit_sexpr.clone());
        assert_eq!(ValueEnum::from(unit_sexpr.clone()), unit_value);
        assert_eq!(
            Sexpr::try_from(unit_value.clone()).expect("Correct variant"),
            unit_sexpr
        );
        assert_eq!(
            <&Sexpr>::try_from(&unit_value).expect("Correct variant"),
            &unit_sexpr
        );
        assert_eq!(NormalValue::from(unit_sexpr), NormalValue::from(()));
        assert_eq!(NormalValue::from(unit_value), NormalValue::from(()));
    }
    /// Test converting simple singleton S-expressions to and from ValueEnum/NormalValue works properly
    #[test]
    fn singleton_value_construction() {
        let st = Sexpr::singleton(true.into());
        let stv = ValueEnum::Sexpr(st.clone());
        assert_eq!(ValueEnum::from(st.clone()), stv);
        assert_eq!(Sexpr::try_from(stv.clone()).expect("Correct variant"), st);
        assert_eq!(<&Sexpr>::try_from(&stv).expect("Correct variant"), &st);
        assert_eq!(NormalValue::from(st), NormalValue::from(true));
        assert_eq!(NormalValue::from(stv), NormalValue::from(true));
    }
}
