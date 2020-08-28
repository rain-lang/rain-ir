/*!
Bitvector addition
*/

use super::*;

lazy_static! {
    /// The addition operator constant
    static ref ADD: VarId<Add> = VarId::direct_new(Add);
}

/// The addition operator
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Add;

/// Perform wrapping bitvector addition, discarding bits of order greater than `len`
#[inline(always)]
pub fn masked_add(len: u32, left: u128, right: u128) -> u128 {
    mask(len, left.wrapping_add(right))
}

debug_from_display!(Add);
quick_pretty!(Add, "Add(Need to change this)");
trivial_substitute!(Add);
enum_convert! {
    impl InjectionRef<ValueEnum> for Add {}
    impl TryFrom<NormalValue> for Add { as ValueEnum, }
    impl TryFromRef<NormalValue> for Add { as ValueEnum, }
}

impl From<Add> for NormalValue {
    fn from(a: Add) -> NormalValue {
        a.into_norm()
    }
}

impl Regional for Add {}

impl Apply for Add {
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
                if b.data == 0 {
                    return Ok(Application::Success(&[], Lambda::id(args[0].clone().coerce()).into_val()));
                }
            }
        }
        if args.len() <= 2 {
            BITS_BINARY
                .apply_ty_in(args, ctx)
                .map(Application::Symbolic)
        } else if args.len() > 3 {
            Err(Error::TooManyArgs)
        } else {
            match (args[0].as_enum(), args[1].as_enum(), args[2].as_enum()) {
                (ValueEnum::BitsTy(ty), ValueEnum::Bits(left), ValueEnum::Bits(right)) => {
                    if left.len != right.len || left.len != ty.0 {
                        return Err(Error::TypeMismatch);
                    }
                    let result = Bits {
                        ty: left.ty.clone(),
                        data: masked_add(ty.0, left.data, right.data),
                        len: left.len,
                    };
                    Ok(Application::Success(&[], result.into_val()))
                }
                (ValueEnum::BitsTy(ty), ValueEnum::Bits(zero), x) if zero.data == 0 => {
                    if zero.len != ty.0 || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    Ok(Application::Success(&[], args[2].clone()))
                }
                (ValueEnum::BitsTy(ty), x, ValueEnum::Bits(zero)) if zero.data == 0 => {
                    if zero.len != ty.0 || zero.ty != x.ty() {
                        return Err(Error::TypeMismatch);
                    }
                    Ok(Application::Success(&[], args[1].clone()))
                }
                (ty, left, right) => {
                    let left_ty = left.ty();
                    if left_ty != right.ty() || left_ty != args[0] || ty.ty() != *BITS_KIND {
                        Err(Error::TypeMismatch)
                    } else {
                        Ok(Application::Symbolic(args[0].clone().coerce()))
                    }
                }
            }
        }
    }
}

impl Typed for Add {
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

impl Type for Add {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Value for Add {
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
        ValueEnum::Add(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Add(self))
    }
}

impl ValueData for Add {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_bitvector_addition_works() {
        let test_cases: &[(u32, u128, u128, u128)] = &[
            // Basic test cases, with no overflow
            (2, 1, 2, 3),
            (4, 3, 4, 7),
            // Overflow tests
            (2, 3, 3, 6 % 4),
            (7, 127, 1, 0),
            (8, 255, 1, 0),
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
            match Add.apply_in(&data_arr[..], &mut ctx).unwrap() {
                Application::Success(&[], v) => match v.as_enum() {
                    ValueEnum::Bits(b) => {
                        assert_eq!(b.len, *len);
                        assert_eq!(b.data, *result);
                        assert_eq!(b.data, mask(*len, left.wrapping_add(*right)));
                        assert_eq!(b.data, masked_add(*len, *left, *right));
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
