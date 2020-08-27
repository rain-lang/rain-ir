use super::*;

/// The negation operator
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Neg {
    /// Type of the negation operator
    ty: VarId<Pi>,
    /// The length of the bit vector,
    len: u32,
}

#[allow(clippy::len_without_is_empty)]
impl Neg {
    /// Create an negation operator with bitwidth `len`
    pub fn new(len: u32) -> Neg {
        Neg {
            ty: Self::compute_ty(len).into_var(),
            len,
        }
    }
    /// Get the pi type of the negation operator with bitwidth `len`
    ///
    /// Note that the result of this method called on `len` is always equal to the type of `Mul::new(len)`.
    pub fn compute_ty(len: u32) -> Pi {
        let region = Region::with_unchecked(
            tyarr![BitsTy{0: len}.into_ty(); 2],
            Region::NULL,
            Fin.into_universe(),
        );
        Pi::try_new(BitsTy(len).into_ty(), region)
            .expect("The type of the multiply operator is always valid")
    }
    /// Perform wrapping bitvector negation, discarding high order bits
    ///
    /// This method assumes both `b` are valid bitvectors for this negation operation, namely that they have length
    /// less than or equal to `self.len()`. If this is not the case, this function will panic *in debug mode*, while in release mode,
    /// the behaviour is unspecified but safe.
    #[inline(always)]
    pub fn masked_neg(&self, b: u128) -> u128 {
        debug_assert_eq!(
            b,
            mask(self.len, b),
            "Bitvector for negation has length greater than len"
        );
        masked_neg(self.len, b)
    }
    /// Get the bitwidth of this addition operator
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.len
    }
}

/// Perform bitvector negation, discarding bits of order greater than `len`
#[inline(always)]
pub fn masked_neg(len: u32, b: u128) -> u128 {
    mask(len, b.wrapping_neg())
}

debug_from_display!(Neg);
quick_pretty!(Neg, "Mul(Need to change this)");
trivial_substitute!(Neg);
enum_convert! {
    impl InjectionRef<ValueEnum> for Neg {}
    impl TryFrom<NormalValue> for Neg { as ValueEnum, }
    impl TryFromRef<NormalValue> for Neg { as ValueEnum, }
}

impl From<Neg> for NormalValue {
    fn from(a: Neg) -> NormalValue {
        a.into_norm()
    }
}

impl Regional for Neg {}

impl Apply for Neg {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        if args.is_empty() {
            self.ty.apply_ty_in(args, ctx).map(Application::Symbolic)
        } else if args.len() > 1 {
            Err(Error::TooManyArgs)
        } else {
            let arg = &args[0];
            match arg.as_enum() {
                ValueEnum::Bits(b) => {
                    let result = Bits {
                        ty: b.ty.clone(),
                        data: self.masked_neg(b.data),
                        len: b.len,
                    };
                    Ok(Application::Success(&[], result.into_val()))
                }
                ValueEnum::BitsTy(b) if b.0 == self.len => {
                    Ok(Application::Symbolic(b.ty().clone_ty()))
                }
                _ => Err(Error::TypeMismatch),
            }
        }
    }
}

impl Typed for Neg {
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

impl Type for Neg {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Value for Neg {
    fn no_deps(&self) -> usize {
        0
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Add operation {} has no dependencies (tried to get dep #{})",
            self, ix
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Neg(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Neg(self))
    }
}

impl ValueData for Neg {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_bitvector_negation_works() {
        let test_cases: &[(u32, u128, u128)] = &[(4, 2, 14), (4, 0, 0), (10, 1, 1023)];
        for (len, data, result) in test_cases.iter() {
            let op_data = BitsTy(*len).data(*data).expect("data is valid");
            let neg_struct = Neg::new(*len);
            let data_arr = [op_data.into_val()];
            let mut ctx = None;
            match neg_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, data.wrapping_neg()));
                        assert_eq!(b.data, masked_neg(*len, *data));
                        assert_eq!(b.data, neg_struct.masked_neg(*data));
                    }
                    _ => panic!("Result should be a bitvector constant (ValueEnum::Bits)"),
                },
                _ => panic!("Application should not be symbolic"),
            };
            assert_eq!(
                ctx, None,
                "No evaluation context should be generated by direct addition of constants"
            );
        }
    }
}
