use super::*;

/// The multiplication operator
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Mul {
    /// Type of the multiply operator
    ty: VarId<Pi>,
    /// The length of the bit vector,
    len: u32,
}

#[allow(clippy::len_without_is_empty)]
impl Mul {
    /// Create a multiply operator with bitwidth `len`
    pub fn new(len: u32) -> Mul {
        Mul {
            ty: Self::compute_ty(len).into_var(),
            len,
        }
    }
    /// Get the pi type of the multiplication operator with bitwidth `len`
    ///
    /// Note that the result of this method called on `len` is always equal to the type of `Mul::new(len)`.
    pub fn compute_ty(len: u32) -> Pi {
        let region = Region::with_unchecked(
            tyarr![BitsTy{0: len}.into_ty(); 2],
            Region::NULL,
            Fin.into_universe(),
        );
        Pi::try_new(BitsTy(len).into_ty(), region)
            .expect("The type of the multiplication operator is always valid")
    }
    /// Perform wrapping bitvector multiplication, discarding high order bits
    ///
    /// This method assumes both `left` and `right` are valid bitvectors for this multiplication operation, namely that they have length
    /// less than or equal to `self.len()`. If this is not the case, this function will panic *in debug mode*, while in release mode,
    /// the behaviour is unspecified but safe.
    #[inline(always)]
    pub fn masked_mul(&self, left: u128, right: u128) -> u128 {
        debug_assert_eq!(
            left,
            mask(self.len, left),
            "Left bitvector has length greater than len"
        );
        debug_assert_eq!(
            right,
            mask(self.len, right),
            "Right bitvector has length greater than len"
        );
        masked_mul(self.len, left, right)
    }
    /// Get the bitwidth of this addition operator
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.len
    }
}

/// Perform wrapping bitvector multiplication, discarding bits of order greater than `len`
#[inline(always)]
pub fn masked_mul(len: u32, left: u128, right: u128) -> u128 {
    mask(len, left.wrapping_mul(right))
}

debug_from_display!(Mul);
quick_pretty!(Mul, "Mul(Need to change this)");
trivial_substitute!(Mul);
enum_convert! {
    impl InjectionRef<ValueEnum> for Mul {}
    impl TryFrom<NormalValue> for Mul { as ValueEnum, }
    impl TryFromRef<NormalValue> for Mul { as ValueEnum, }
}

impl From<Mul> for NormalValue {
    fn from(a: Mul) -> NormalValue {
        a.into_norm()
    }
}

impl Regional for Mul {}

impl Apply for Mul {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        if args.len() <= 1 {
            self.ty.apply_ty_in(args, ctx).map(Application::Symbolic)
        } else if args.len() > 2 {
            Err(Error::TooManyArgs)
        } else {
            let arg_0 = &args[0];
            let arg_1 = &args[1];
            match (arg_0.as_enum(), arg_1.as_enum()) {
                (ValueEnum::Bits(left), ValueEnum::Bits(right)) => {
                    if left.len != right.len || left.len != self.len {
                        return Err(Error::TypeMismatch);
                    }
                    let result = Bits {
                        ty: left.ty.clone(),
                        data: self.masked_mul(left.data, right.data),
                        len: left.len,
                    };
                    Ok(Application::Success(&[], result.into_val()))
                }
                (ValueEnum::Bits(one), x) if one.data == 1 => {
                    if one.len != self.len || one.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    Ok(Application::Success(&[], args[0].clone()))
                }
                (x, ValueEnum::Bits(one)) if one.data == 1 => {
                    if one.len != self.len || one.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    Ok(Application::Success(&[], args[1].clone()))
                }
                (ValueEnum::Bits(zero), x) | (x, ValueEnum::Bits(zero)) if zero.data == 0 => {
                    if zero.len != self.len || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    let result = BitsTy { 0: zero.len }.data(0).unwrap();
                    Ok(Application::Success(&[], result.into_val()))
                }
                (left, right) => {
                    let left_ty = left.ty();
                    if left_ty != right.ty() {
                        // Error
                        return Err(Error::TypeMismatch);
                    }
                    match left {
                        ValueEnum::BitsTy(b) if b.0 != self.len => {
                            Ok(Application::Symbolic(left_ty.clone_as_ty()))
                        }
                        _ => Err(Error::TypeMismatch),
                    }
                }
            }
        }
    }
}

impl Typed for Mul {
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

impl Type for Mul {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Value for Mul {
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
        ValueEnum::Mul(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Mul(self))
    }
}

impl ValueData for Mul {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_bitvector_multiplication_works() {
        let test_cases: &[(u32, u128, u128, u128)] = &[
            // Basic test cases, with no overflow
            (2, 1, 2, 2),
            (4, 3, 4, 12),
            (4, 3, 1, 3),
            (4, 3, 0, 0),
            // Overflow tests
            (2, 3, 3, 9 % 4),
            (7, 127, 2, (127 * 2) % 128),
            (14, 8848, 2, 1312),
        ];
        for (len, left, right, result) in test_cases.iter() {
            let left_data = BitsTy(*len).data(*left).expect("Left data is valid");
            let right_data = BitsTy(*len).data(*right).expect("Right data is valid");
            let multiply_struct = Mul::new(*len);
            let data_arr = [left_data.into_val(), right_data.into_val()];
            let mut ctx = None;
            match multiply_struct.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_mul(*right)));
                        assert_eq!(b.data, masked_mul(*len, *left, *right));
                        assert_eq!(b.data, multiply_struct.masked_mul(*left, *right));
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
