/*!
Miscellaneous utilities and data structures used throughout the `rain` compiler
*/
mod addr;
pub use addr::*;
mod format;

/// Implement `From<T> for E` and implement `TryFrom<E> for T>` where `T` is an enum variant of `E`
#[macro_export]
macro_rules! enum_convert {
    (impl From<$T:ty> for $E:ty { as $En:ident$(::$V:ident)+, }) => {
        impl From<$T> for $E {
            fn from(v: $T) -> $E { $En::$($V)+(v) }
        }
    };
    (impl From<$T:ident> for $E:ty { as $En:ident, }) => {
        enum_convert!(
            impl From<$T> for $E { as $En::$T, }
        );
    };
    (impl From<$T:ident> for $E:ident {}) => {
        enum_convert!(
            impl From<$T> for $E { as $E, }
        );
    };
    (impl TryFrom<$E:ty> for $T:ty { as $En:ident$(::$V:ident)+, $(match $($from:pat $(if $guard:expr)* => $to:expr,)*)* }) => {
        impl std::convert::TryFrom<$E> for $T {
            type Error = $E;
            fn try_from(v: $E) -> Result<$T, $E> {
                use std::borrow::Borrow;
                let v_ref: &$En = v.borrow();
                #[allow(unreachable_patterns, unused_variables)]
                match v_ref {
                    $En$(::$V)+(_) => match v.into() {
                        $En$(::$V)+(v) => Ok(v),
                        _ => panic!("Impossible: pattern was previously matched on reference")
                    },
                    $($($from $(if $guard)* => {
                        let v: $En = v.into();
                        match v {
                            $from => $to,
                            _ => panic!("Impossible: guarded pattern was previously matched on reference")
                        }
                    },)*)*
                    _ => Err(v)
                }
            }
        }
    };
    (impl TryFrom<$E:ty> for $T:ident { as $En:ident, $(match $($from:pat $(if $guard:expr)* => $to:expr,)*)* }) => {
        enum_convert! {
            impl TryFrom<$E> for $T { as $En::$T, $(match $($from $(if $guard)* => $to,)*)* }
        }
    };
    (impl TryFrom<$E:ident> for $T:ident { $(match $($from:pat $(if $guard:expr)* => $to:expr,)*)* }) => {
        enum_convert! {
            impl TryFrom<$E> for $T { as $E::$T, $(match $($from $(if $guard)* => $to,)*)* }
        }
    };
    (impl TryFrom<$E:ident> for $T:ident {}) => {
        enum_convert!{
            impl TryFrom<$E> for $T { as $E, }
        }
    };
    (impl TryFromRef<$E:ty> for $T:ty { as $En:ident$(::$V:ident)+, $(match $($from:pat $(if $guard:expr)* => $to:expr,)*)* }) => {
        impl<'a> std::convert::TryFrom<&'a $E> for &'a $T {
            type Error = &'a $E;
            fn try_from(v: &'a $E) -> Result<&'a $T, &'a $E> {
                use std::borrow::Borrow;
                let v_ref: &$En = v.borrow();
                #[allow(unreachable_patterns, unused_variables)]
                match v_ref {
                    $En$(::$V)+(r) => Ok(r),
                    $($($from $(if $guard)* => $to,)*)*
                    _ => Err(v)
                }
            }
        }
    };
    (impl TryFromRef<$E:ty> for $T:ident { as $En:ident, $(match $($from:pat $(if $guard:expr)* => $to:expr,)*)* }) => {
        enum_convert! {
            impl TryFromRef<$E> for $T { as $En::$T, $(match $($from $(if $guard)* => $to,)*)* }
        }
    };
    (impl TryFromRef<$E:ident> for $T:ident { $(match $($from:pat $(if $guard:expr)* => $to:expr,)*)* }) => {
        enum_convert! {
            impl TryFromRef<$E> for $T { as $E::$T, $(match $($from $(if $guard)* => $to,)*)* }
        }
    };
    (impl TryFromRef<$E:ident> for $T:ident {}) => {
        enum_convert!{
            impl TryFromRef<$E> for $T { as $E, }
        }
    };
    (impl Injection<$E:ident> for $T:ident { $(as $t:ident,)* $(match $($from:pat $(if $guard:expr)* => $to:expr,)*)* }) => {
        enum_convert!{
            impl From<$T> for $E { $(as $t,)* }
            impl TryFrom<$E> for $T { $(as $t,)* $(match $($from $(if $guard)* => $to,)*)* }
        }
    };
    (impl InjectionRef<$E:ident> for $T:ident { $(as $t:ident,)* $(match $($from:pat $(if $guard:expr)* => $to:expr,)*)* }) => {
        enum_convert!{
            impl From<$T> for $E { $(as $t,)* }
            impl TryFrom<$E> for $T { $(as $t,)* $(match $($from $(if $guard)* => $to,)*)* }
            impl TryFromRef<$E> for $T { $(as $t,)* $(match $($from $(if $guard)* => $to,)*)* }
        }
    };
    (
        impl $Tr_first:ident<$E_first:ident> for $T_first:ident {
            $(as $t_first:ident,)*
            $(match $($from_first:pat $(if $guard_first:expr)* => $to_first:expr,)*)*
        }
        $(impl $Tr:ident<$E:ident> for $T:ident {
            $(as $t:ident,)*
            $(match $($from:pat $(if $guard:expr)* => $to:expr,)*)*
        })+
    ) => {
        enum_convert! {
            impl $Tr_first<$E_first> for $T_first {
                $(as $t_first,)*
                $(match $($from_first $(if $guard_first)* => $to_first,)*)*
            }
        }
        $(
            enum_convert!(
                impl $Tr<$E> for $T { $(as $t,)* $(match $($from $(if $guard)* => $to,)*)* }
            );
        )+
    }
}