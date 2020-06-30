/*!
Tuples of `rain` values and their associated finite (Cartesian) product types
*/
use super::{
    arr::{TyArr, ValArr},
    universe::FINITE_TY,
    Error, NormalValue, TypeId, TypeRef, UniverseId, UniverseRef, ValId, Value, ValueData,
    ValueEnum,
};
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::primitive::{Unit, UNIT_TY};
use crate::typing::{Type, Typed};
use crate::{debug_from_display, lifetime_region, pretty_display, substitute_to_valid};
use std::convert::TryInto;
use std::ops::Deref;

/// A tuple of `rain` values
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Tuple {
    /// The elements of this tuple
    elems: ValArr,
    /// The (cached) lifetime of this tuple
    lifetime: Lifetime,
    /// The (cached) type of this tuple
    ty: TypeId,
}

impl Tuple {
    /// Try to create a new product from a vector of values. Return an error if they have incompatible lifetimes.
    #[inline]
    pub fn try_new(elems: ValArr) -> Result<Tuple, ()> {
        let lifetime = Lifetime::default().intersect(elems.iter().map(|t| t.lifetime()))?;
        let ty = Product::try_new(elems.iter().map(|elem| elem.ty().clone_ty()).collect())?.into();
        Ok(Tuple {
            elems,
            lifetime,
            ty,
        })
    }
    /// Create the tuple corresponding to the element of the unit type
    #[inline]
    pub fn unit() -> Tuple {
        Tuple {
            elems: ValArr::EMPTY,
            lifetime: Lifetime::default(),
            ty: UNIT_TY.as_ty().clone(),
        }
    }
}

impl Live for Tuple {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

lifetime_region!(Tuple);

impl Deref for Tuple {
    type Target = ValArr;
    #[inline]
    fn deref(&self) -> &ValArr {
        &self.elems
    }
}

impl Typed for Tuple {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
}

impl Value for Tuple {
    #[inline]
    fn no_deps(&self) -> usize {
        self.len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self[ix]
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Tuple(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for Tuple {}

impl Apply for Tuple {
    /**
    Tuples accept finite indices as arguments, which is the `rain` syntax for a member access.
    */
    fn do_apply<'a>(&self, args: &'a [ValId], _inline: bool) -> Result<Application<'a>, Error> {
        // Check for a null application
        if args.len() == 0 {
            return Ok(Application::Complete(
                self.lifetime().clone_lifetime(),
                self.ty().clone_ty(),
            ));
        }
        // Do a type check
        match args[0].ty().as_enum() {
            ValueEnum::Finite(f) => {
                if self.len() as u128 != f.0 {
                    return Err(Error::TupleLengthMismatch);
                }
            }
            _ => return Err(Error::TypeMismatch),
        }
        // See if we can actually evaluate this expression
        match args[0].as_enum() {
            ValueEnum::Index(ix) => Ok(Application::Success(
                &args[1..],
                self[ix.ix() as usize].clone(),
            )),
            _ => unimplemented!(), //TODO: product downcasting...
        }
    }
}

impl Substitute for Tuple {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Tuple, Error> {
        let lifetime = ctx.evaluate_lt(&self.lifetime)?;
        let elems = self
            .elems
            .iter()
            .cloned()
            .map(|val| val.substitute(ctx))
            .collect::<Result<_, _>>()?;
        Ok(Tuple {
            elems,
            lifetime,
            ty: self.ty.substitute(ctx)?,
        })
    }
}

substitute_to_valid!(Tuple);

impl From<Tuple> for NormalValue {
    fn from(tuple: Tuple) -> NormalValue {
        if tuple == () {
            return ().into();
        }
        NormalValue(ValueEnum::Tuple(tuple))
    }
}

debug_from_display!(Tuple);
pretty_display!(Tuple, "[...]");

/// A product of `rain` values
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Product {
    /// The elements of this product type
    elems: TyArr,
    /// The (cached) lifetime of this product type
    lifetime: Lifetime,
    /// The (cached) type of this product type
    ty: UniverseId,
}

impl Product {
    /// Try to create a new product from a vector of types. Return an error if they have incompatible lifetimes.
    #[inline]
    pub fn try_new(elems: TyArr) -> Result<Product, ()> {
        let lifetime = Lifetime::default().intersect(elems.iter().map(|t| t.lifetime()))?;
        let ty = FINITE_TY.union_all(elems.iter().map(|t| t.universe()));
        Ok(Product {
            elems,
            lifetime,
            ty,
        })
    }
    /// Create the product corresponding to the unit type
    #[inline]
    pub fn unit_ty() -> Product {
        Product {
            elems: TyArr::EMPTY,
            lifetime: Lifetime::default(),
            ty: FINITE_TY.clone(),
        }
    }
    /// Get the type-tuple corresponding to this product type
    ///
    /// TODO: consider caching this (or the tuple type) in an atomic, as it may need to be computed many times
    #[inline]
    pub fn tuple(&self) -> Tuple {
        let ty_elems = self.elems.iter().map(|elem| elem.ty().clone_ty()).collect();
        //TODO: think about this...
        let ty = Product::try_new(ty_elems).expect("Impossible").into();
        Tuple {
            elems: self.elems.as_vals().clone(),
            lifetime: self.lifetime.clone(),
            ty,
        }
    }
}

debug_from_display!(Product);
pretty_display!(Product, "#product [...]");

impl Substitute for Product {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Product, Error> {
        let lifetime = ctx.evaluate_lt(&self.lifetime)?;
        let elems = self
            .elems
            .iter()
            .cloned()
            .map(|val| -> Result<TypeId, _> { val.substitute(ctx) })
            .collect::<Result<_, _>>()?;
        Ok(Product {
            elems,
            lifetime,
            ty: self.ty.substitute(ctx)?.try_into().expect("Impossible"),
        })
    }
}

substitute_to_valid!(Product);

impl Live for Product {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

lifetime_region!(Product);

impl Deref for Product {
    type Target = TyArr;
    #[inline]
    fn deref(&self) -> &TyArr {
        &self.elems
    }
}

impl Typed for Product {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Apply for Product {}

impl Type for Product {
    fn universe(&self) -> UniverseRef {
        self.ty.borrow_var()
    }
    fn is_universe(&self) -> bool {
        false
    }
}

impl From<Product> for NormalValue {
    fn from(product: Product) -> NormalValue {
        if product == Unit {
            return Unit.into();
        }
        NormalValue(ValueEnum::Product(product))
    }
}

impl Value for Product {
    #[inline]
    fn no_deps(&self) -> usize {
        self.len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        (&self[ix]).into()
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Product(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for Product {}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{tokens::*, PrettyPrint, PrettyPrinter};
    use crate::primitive::Unit;
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Tuple {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            if self.len() == 0 {
                return write!(fmt, "{}", UNIT_VALUE);
            }
            write!(fmt, "{}", TUPLE_OPEN)?;
            let mut first = true;
            for elem in self.iter() {
                if !first {
                    write!(fmt, " ")?;
                }
                first = false;
                elem.prettyprint(printer, fmt)?;
            }
            write!(fmt, "{}", TUPLE_CLOSE)
        }
    }

    impl PrettyPrint for Product {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            if self.len() == 0 {
                return write!(fmt, "{}", Unit);
            }
            write!(fmt, "{}{}", KEYWORD_PROD, TUPLE_OPEN)?;
            let mut first = true;
            for elem in self.iter() {
                if !first {
                    write!(fmt, " ")?;
                }
                first = false;
                elem.prettyprint(printer, fmt)?;
            }
            write!(fmt, "{}", TUPLE_CLOSE)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::parser::builder::Builder;
        use crate::valarr;

        #[test]
        fn nested_units_print_properly() {
            let unit = Tuple::unit();
            let unit_ty = Product::unit_ty();
            assert_eq!(format!("{}", unit), format!("{}", UNIT_VALUE));
            assert_eq!(format!("{}", unit_ty), format!("{}", Unit));
            let two_units = Tuple::try_new(valarr![unit.clone().into(), unit.into()])
                .expect("This is a valid tuple!");
            assert_eq!(
                format!("{}", two_units),
                format!("{}{} {}{}", TUPLE_OPEN, UNIT_VALUE, UNIT_VALUE, TUPLE_CLOSE)
            );
            let unit_squared = two_units.ty();
            assert_eq!(
                format!("{}", unit_squared),
                format!(
                    "{}{}{} {}{}",
                    KEYWORD_PROD, TUPLE_OPEN, Unit, Unit, TUPLE_CLOSE
                )
            );
        }

        #[test]
        fn simple_projections_normalize_properly() {
            let mut builder = Builder::<&str>::new();
            assert_eq!(
                builder.parse_expr("[#true #false].0").unwrap(),
                ("", ValId::from(true))
            );
            assert_eq!(
                builder.parse_expr("[#true #false].1").unwrap(),
                ("", ValId::from(false))
            );
            assert_eq!(
                builder
                    .parse_expr("[[#true #false] [#false #true] []].1.0")
                    .unwrap(),
                ("", ValId::from(false))
            );
        }
    }
}
