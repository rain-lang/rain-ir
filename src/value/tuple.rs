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
use crate::{
    debug_from_display, enum_convert, lifetime_region, pretty_display, substitute_to_valid,
};
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
    pub fn try_new(elems: ValArr) -> Result<Tuple, Error> {
        let lifetime = Lifetime::default().sep_conj(elems.iter().map(|t| t.lifetime()))?;
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
    /// Create a *constant* "anchor", i.e. a tuple corresponding to the element of the unit type where the unit type is marked affine
    #[inline]
    pub fn const_anchor() -> Tuple {
        Tuple {
            elems: ValArr::EMPTY,
            lifetime: Lifetime::default(),
            ty: Product::anchor_ty().into(),
        }
    }
    /// Check whether this tuple is an anchor
    #[inline]
    pub fn is_anchor(&self) -> bool {
        match self.ty.as_enum() {
            ValueEnum::Product(p) => p.is_anchor(),
            _ => false,
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
    #[inline]
    fn cast(self, ty: Option<TypeId>, lt: Option<Lifetime>) -> Result<ValId, Error> {
        if ty.is_none() && lt.is_none() {
            return Ok(self.into_val());
        }
        let lt = if let Some(lt) = lt {
            self.cast_target_lt(lt)?
        } else {
            self.lifetime().clone_lifetime()
        };
        let ty = if let Some(ty) = ty {
            self.cast_target_ty(ty)?
        } else {
            self.ty().clone_ty()
        };
        if lt == self.lifetime() && ty == self.ty() {
            return Ok(self.into_val());
        }
        Ok(NormalValue(ValueEnum::Tuple(Tuple {
            elems: self.elems,
            ty,
            lifetime: lt,
        }))
        .into())
    }
}

impl ValueData for Tuple {}

impl Apply for Tuple {
    /**
    Tuples accept finite indices as arguments, which is the `rain` syntax for a member access.
    */
    fn do_apply<'a>(&self, args: &'a [ValId], _inline: bool) -> Result<Application<'a>, Error> {
        // Check for a null application
        if args.is_empty() {
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
enum_convert! {
    impl InjectionRef<ValueEnum> for Tuple {}
    impl TryFrom<NormalValue> for Tuple { as ValueEnum, }
    impl TryFromRef<NormalValue> for Tuple { as ValueEnum, }
}

/// The set of flags for a product of rain values
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
struct ProductFlags(u8);

impl ProductFlags {
    #[inline]
    pub fn new(affine: bool, anchor: bool, relevant: bool, flare: bool) -> ProductFlags {
        let affine = affine as u8 * FLAG_AFFIN;
        let anchor = anchor as u8 * FLAG_ANCHR;
        let relevant = relevant as u8 * FLAG_RLVNT;
        let shiny = flare as u8 * FLAG_FLARE;
        ProductFlags(affine | anchor | relevant | shiny)
    }
    #[inline]
    pub fn is_affine(self) -> bool {
        self.0 & FLAG_AFFIN != 0
    }
    #[inline]
    pub fn is_anchor(self) -> bool {
        self.0 & FLAG_ANCHR != 0
    }
    #[inline]
    pub fn is_relevant(self) -> bool {
        self.0 & FLAG_RLVNT != 0
    }
    #[inline]
    pub fn is_flare(self) -> bool {
        self.0 & FLAG_FLARE != 0
    }
}

const FLAG_AFFIN: u8 = 0b00000001;
const FLAG_ANCHR: u8 = 0b00000010;
const FLAG_RLVNT: u8 = 0b00000100;
const FLAG_FLARE: u8 = 0b00001000;

/// A product of `rain` values
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Product {
    /// The elements of this product type
    elems: TyArr,
    /// The (cached) lifetime of this product type
    lifetime: Lifetime,
    /// The (cached) type of this product type
    ty: UniverseId,
    /// The flags on this product type
    flags: ProductFlags,
}

impl Product {
    /// Try to create a new product from a vector of types, potentially forcing affinity/relevancy
    /// Return an error if they have incompatible lifetimes.
    #[inline]
    pub fn try_new_forced(
        elems: TyArr,
        force_affine: bool,
        force_relevant: bool,
    ) -> Result<Product, Error> {
        let lifetime = Lifetime::default().sep_conj(elems.iter().map(|t| t.lifetime()))?;
        let affine = force_affine || elems.iter().any(|t| t.is_affine());
        let relevant = force_relevant || elems.iter().any(|t| t.is_relevant());
        let flags = ProductFlags::new(affine, force_affine, relevant, force_relevant);
        let ty = FINITE_TY.union_all(elems.iter().map(|t| t.universe()));
        Ok(Product {
            elems,
            lifetime,
            ty,
            flags,
        })
    }
    /// Try to create a new product from a vector of types. Return an error if they have incompatible lifetimes.
    #[inline]
    pub fn try_new(elems: TyArr) -> Result<Product, Error> {
        Self::try_new_forced(elems, false, false)
    }
    /// Create the product corresponding to the unit type
    #[inline]
    pub fn unit_ty() -> Product {
        Product {
            elems: TyArr::EMPTY,
            lifetime: Lifetime::default(),
            ty: FINITE_TY.clone(),
            flags: ProductFlags(0),
        }
    }
    /// Create the product corresponding to the "anchor" type, i.e. the unit type made affine
    #[inline]
    pub fn anchor_ty() -> Product {
        Product {
            elems: TyArr::EMPTY,
            lifetime: Lifetime::default(),
            ty: FINITE_TY.clone(),
            flags: ProductFlags(FLAG_AFFIN | FLAG_ANCHR),
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
    /// Get whether this product type is an anchor, i.e. forcibly affine
    pub fn is_anchor(&self) -> bool {
        self.flags.is_anchor()
    }
    /// Get whether this product type is a flare, i.e. forcibly relevant
    pub fn is_flare(&self) -> bool {
        self.flags.is_flare()
    }
}

debug_from_display!(Product);
pretty_display!(Product, "#product [...]");
enum_convert! {
    impl InjectionRef<ValueEnum> for Product {}
    impl TryFrom<NormalValue> for Product { as ValueEnum, }
    impl TryFromRef<NormalValue> for Product { as ValueEnum, }
}

impl Substitute for Product {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Product, Error> {
        let lifetime = ctx.evaluate_lt(&self.lifetime)?;
        let elems: TyArr = self
            .elems
            .iter()
            .cloned()
            .map(|val| -> Result<TypeId, _> { val.substitute(ctx) })
            .collect::<Result<_, _>>()?;
        let affine = self.is_anchor() || elems.iter().any(|t| t.is_affine());
        let relevant = self.is_flare() || elems.iter().any(|t| t.is_affine());
        let flags = ProductFlags::new(affine, self.is_anchor(), relevant, self.is_flare());
        Ok(Product {
            elems,
            lifetime,
            ty: self.ty.substitute(ctx)?.try_into().expect("Impossible"),
            flags,
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
    #[inline]
    fn is_affine(&self) -> bool {
        self.flags.is_affine()
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        self.flags.is_relevant()
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
    #[inline]
    fn cast(self, ty: Option<TypeId>, lt: Option<Lifetime>) -> Result<ValId, Error> {
        if ty.is_none() && lt.is_none() {
            return Ok(self.into_val());
        }
        let lt = if let Some(lt) = lt {
            self.cast_target_lt(lt)?
        } else {
            self.lifetime().clone_lifetime()
        };
        //TODO: proper universe casting...
        if let Some(ty) = ty {
            if ty != self.ty() {
                return Err(Error::TypeMismatch);
            }
        }
        if lt == self.lifetime() {
            return Ok(self.into_val());
        }
        Ok(NormalValue(ValueEnum::Product(Product {
            elems: self.elems,
            ty: self.ty,
            lifetime: lt,
            flags: self.flags,
        }))
        .into())
    }
}

impl ValueData for Product {}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use crate::primitive::Unit;
    use crate::tokens::*;
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Tuple {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            if *self == () {
                return write!(fmt, "{}", UNIT_VALUE);
            }
            write!(
                fmt,
                "{}{}",
                if self.is_anchor() {
                    KEYWORD_ANCHORED
                } else {
                    ""
                },
                TUPLE_OPEN
            )?;
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
            if *self == Unit {
                return write!(fmt, "{}", Unit);
            }
            write!(
                fmt,
                "{}{}",
                if self.is_anchor() {
                    KEYWORD_ANCHOR
                } else {
                    KEYWORD_PROD
                },
                TUPLE_OPEN
            )?;
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
        use crate::valarr;

        #[test]
        fn nested_units_print_properly() {
            let unit = Tuple::unit();
            let unit_ty = Product::unit_ty();
            assert_eq!(&format!("{}", unit), UNIT_VALUE);
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
            let anchor: ValId = Tuple::const_anchor().into();
            assert_eq!(&format!("{}", anchor), &format!("{}[]", KEYWORD_ANCHORED));
            assert_eq!(
                &format!("{}", anchor.ty()),
                &format!("{}[]", KEYWORD_ANCHOR)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    /// Test converting the unit tuple to and from ValueEnum/NormalValue works properly
    #[test]
    fn unit_value_construction() {
        let unit_tuple = Tuple::unit();
        let unit_value = ValueEnum::Tuple(unit_tuple.clone());
        assert_eq!(ValueEnum::from(unit_tuple.clone()), unit_value);
        assert_eq!(
            Tuple::try_from(unit_value.clone()).expect("Correct variant"),
            unit_tuple
        );
        assert_eq!(
            <&Tuple>::try_from(&unit_value).expect("Correct variant"),
            &unit_tuple
        );
        assert_eq!(NormalValue::from(unit_tuple), NormalValue::from(()));
        assert_eq!(NormalValue::from(unit_value), NormalValue::from(()));
    }

    /// Test converting the unit type to and from ValueEnum/NormalValue works properly
    #[test]
    fn unit_type_construction() {
        let unit_type = Product::unit_ty();
        let unit_type_enum = ValueEnum::Product(unit_type.clone());
        assert_eq!(ValueEnum::from(unit_type.clone()), unit_type_enum);
        assert_eq!(
            Product::try_from(unit_type_enum.clone()).expect("Correct variant"),
            unit_type
        );
        assert_eq!(
            <&Product>::try_from(&unit_type_enum).expect("Correct variant"),
            &unit_type
        );
        assert_eq!(NormalValue::from(unit_type), NormalValue::from(Unit));
        assert_eq!(NormalValue::from(unit_type_enum), NormalValue::from(Unit));
    }

    /// Test the anchor type is affine, but *can* be bundled with itself to make another affine type
    #[test]
    fn anchor_type_construction() {
        let anchor: ValId = Tuple::const_anchor().into();
        let anchor_ty: TypeId = Product::anchor_ty().into();
        assert_eq!(anchor.ty(), anchor_ty);
        assert!(anchor_ty.is_affine());
        assert!(!anchor_ty.is_relevant());
        let anchor_tuple: ValId = Tuple::try_new(vec![anchor.clone(), anchor].into())
            .expect("Two anchors form a valid tuple")
            .into();
        let anchor_product = anchor_tuple.ty();
        assert!(anchor_product.is_affine());
        assert!(!anchor_product.is_relevant());
    }
}
