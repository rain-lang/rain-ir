/*!
Bitvector trait implementation
*/
use super::*;

debug_from_display!(BitsKind);
quick_pretty!(BitsKind, "#bitskind");
trivial_substitute!(BitsKind);
enum_convert! {
    impl InjectionRef<ValueEnum> for BitsKind {}
    impl TryFrom<NormalValue> for BitsKind { as ValueEnum, }
    impl TryFromRef<NormalValue> for BitsKind { as ValueEnum, }
}

impl ValueData for BitsKind {}

impl Typed for BitsKind {
    #[inline]
    fn ty(&self) -> TypeRef {
        SET.borrow_ty()
    }

    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
    #[inline]
    fn is_kind(&self) -> bool {
        true
    }
}

impl Apply for BitsKind {}

impl Live for BitsKind {}

impl Value for BitsKind {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Tried to get dependency #{} of bits kind, which has none",
            ix
        )
    }
    #[inline]
    fn dep_owned(&self, ix: usize) -> bool {
        panic!(
            "{:?} has no dependencies, but attempted to get no #{}",
            self, ix
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::BitsKind(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::BitsKind(self))
    }
}

impl From<BitsKind> for NormalValue {
    fn from(b: BitsKind) -> NormalValue {
        b.into_norm()
    }
}

impl Type for BitsKind {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
}

impl Kind for BitsKind {
    fn id_kind(&self) -> KindId {
        Prop.into_kind()
    }
    fn closure(&self) -> UniverseId {
        Prop.into_universe()
    }
    fn try_closure(&self) -> Option<UniverseRef> {
        Some(FIN.borrow_universe())
    }
}

debug_from_display!(BitsTy);
quick_pretty!(BitsTy, b, fmt => write!(fmt, "#bitsty({})", b.0));
trivial_substitute!(BitsTy);
enum_convert! {
    impl InjectionRef<ValueEnum> for BitsTy {}
    impl TryFrom<NormalValue> for BitsTy { as ValueEnum, }
    impl TryFromRef<NormalValue> for BitsTy { as ValueEnum, }
}

impl ValueData for BitsTy {}

impl Typed for BitsTy {
    #[inline]
    fn ty(&self) -> TypeRef {
        BITS_KIND.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Apply for BitsTy {}

impl Live for BitsTy {}

impl Value for BitsTy {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Tried to get dependency #{} of bits type {}, which has none",
            ix, self
        )
    }
    #[inline]
    fn dep_owned(&self, ix: usize) -> bool {
        panic!(
            "{:?} has no dependencies, but attempted to get no #{}",
            self, ix
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::BitsTy(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::BitsTy(self))
    }
}

impl From<BitsTy> for NormalValue {
    fn from(b: BitsTy) -> NormalValue {
        b.into_norm()
    }
}

impl Type for BitsTy {
    #[inline]
    fn is_affine(&self) -> bool {
        false
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        false
    }
    #[inline]
    fn apply_ty(&self, args: &[ValId]) -> Result<TypeId, Error> {
        match args {
            [] => Ok(self.into_ty()),
            [ix] => {
                if let ValueEnum::Finite(ix) = ix.ty().as_enum() {
                    if ix.0 == self.0 as u128 {
                        return Ok(Bool.into_ty());
                    }
                }
                Err(Error::TypeMismatch)
            }
            [..] => Err(Error::TooManyArgs),
        }
    }
}

debug_from_display!(Bits);
quick_pretty!(Bits, b, fmt => write!(fmt, "{}'h{:x}", b.len, b.data));
trivial_substitute!(Bits);
enum_convert! {
    impl InjectionRef<ValueEnum> for Bits {}
    impl TryFrom<NormalValue> for Bits { as ValueEnum, }
    impl TryFromRef<NormalValue> for Bits { as ValueEnum, }
}

impl Typed for Bits {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }

    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Apply for Bits {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        _ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        match args {
            [] => Ok(Application::Success(&[], self.clone().into_val())),
            [ix] => {
                let ix = ix.as_enum();
                match ix.ty().as_enum() {
                    ValueEnum::Finite(f) if f.0 == self.len as u128 => {}
                    _ => return Err(Error::TypeMismatch),
                }
                match ix {
                    ValueEnum::Index(ix) => Ok(Application::Success(
                        &[],
                        self.bit_zext(ix.ix() as u32).into_val(),
                    )),
                    _ => Ok(Application::Symbolic(Bool.into_ty())),
                }
            }
            [..] => Err(Error::TooManyArgs),
        }
    }
}

impl Live for Bits {}

impl Value for Bits {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!(
            "Tried to get dependency #{} of bits {}, which has none",
            ix, self
        )
    }
    #[inline]
    fn dep_owned(&self, ix: usize) -> bool {
        panic!(
            "{:?} has no dependencies, but attempted to get no #{}",
            self, ix
        )
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Bits(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Bits(self))
    }
}

impl From<Bits> for NormalValue {
    fn from(b: Bits) -> NormalValue {
        b.into_norm()
    }
}
