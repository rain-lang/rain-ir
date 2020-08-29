/*!
Bitvector multiplication
*/
use super::*;

lazy_static! {
    /// The multiplication operator constant
    static ref ADD: VarId<Mul> = VarId::direct_new(Mul);
}

/// The multiplication operator
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Mul;

/// Perform wrapping bitvector multiplication, discarding bits of order greater than `len`
#[inline(always)]
pub fn masked_mul(len: u32, left: u128, right: u128) -> u128 {
    mask(len, left.wrapping_mul(right))
}

debug_from_display!(Mul);
quick_pretty!(Mul, "#mul");
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
        if args.len() == 2 {
            if let ValueEnum::Bits(b) = args[1].as_enum() {
                if b.ty != args[0] {
                    return Err(Error::TypeMismatch);
                }
                if b.data == 1 {
                    return Ok(Application::Success(
                        &[],
                        Lambda::id(args[0].clone().coerce()).into_val(),
                    ));
                }
            }
        }
        if args.len() <= 2 {
            BITS_BINARY
                .apply_ty_in(args, ctx)
                .map(Application::Symbolic)
        } else {
            match (args[0].as_enum(), args[1].as_enum(), args[2].as_enum()) {
                (ValueEnum::BitsTy(ty), ValueEnum::Bits(left), ValueEnum::Bits(right)) => {
                    if left.len != right.len || left.len != ty.0 {
                        return Err(Error::TypeMismatch);
                    }
                    let result = Bits {
                        ty: left.ty.clone(),
                        data: masked_mul(ty.0, left.data, right.data),
                        len: left.len,
                    };
                    result.apply_in(&args[3..], ctx)
                }
                // Multiplication by zero yields zero
                (ValueEnum::BitsTy(ty), x, ValueEnum::Bits(zero)) if zero.data == 0 => {
                    if zero.len != ty.0 || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    args[2].apply_in(&args[3..], ctx)
                }
                (ValueEnum::BitsTy(ty), ValueEnum::Bits(zero), x) if zero.data == 0 => {
                    if zero.len != ty.0 || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    args[1].apply_in(&args[3..], ctx)
                }
                // Multiplication by one is the identity
                (ValueEnum::BitsTy(ty), ValueEnum::Bits(one), x) if one.data == 1 => {
                    if one.len != ty.0 || one.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    args[2].apply_in(&args[3..], ctx)
                }
                (ValueEnum::BitsTy(ty), x, ValueEnum::Bits(one)) if one.data == 1 => {
                    if one.len != ty.0 || one.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    args[1].apply_in(&args[3..], ctx)
                }
                (ty, left, right) => {
                    let left_ty = left.ty();
                    if left_ty != right.ty() || left_ty != args[0] || ty.ty() != *BITS_KIND {
                        Err(Error::TypeMismatch)
                    } else {
                        left_ty
                            .apply_ty_in(&args[3..], ctx)
                            .map(Application::Symbolic)
                    }
                }
            }
        }
    }
}

impl Typed for Mul {
    #[inline]
    fn ty(&self) -> TypeRef {
        BITS_BINARY.borrow_ty()
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
            "Mul operation {} has no dependencies (tried to get dep #{})",
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
            let bitwidth = BitsTy(*len).into_var();
            let left_data = bitwidth.data(*left).expect("Left data is valid");
            let right_data = bitwidth.data(*right).expect("Right data is valid");
            let data_arr = [
                bitwidth.into_val(),
                left_data.into_val(),
                right_data.into_val(),
            ];
            let mut ctx = None;
            // Complete application
            match Mul.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_mul(*right)));
                        assert_eq!(b.data, masked_mul(*len, *left, *right));
                    }
                    _ => panic!("Result should be a bitvector constant (ValueEnum::Bits)"),
                },
                _ => panic!("Application should not be symbolic"),
            };
            assert_eq!(
                ctx, None,
                "No evaluation context should be generated by direct multiplication of constants"
            );
            /*
            // Nested partial application
            match Mul
                .applied_in(&data_arr[..2], &mut ctx)
                .unwrap()
                .applied_in(&data_arr[2..], &mut ctx)
                .unwrap()
                .as_enum()
            {
                ValueEnum::Bits(b) => {
                    assert_eq!(b.len, *len);
                    assert_eq!(b.data, *result);
                    assert_eq!(b.data, mask(*len, left.wrapping_mul(*right)));
                    assert_eq!(b.data, masked_mul(*len, *left, *right));
                }
                r => panic!("Result should be a bitvector constant (ValueEnum::Bits), but got {}", r),
            };
            */
        }
    }
}
