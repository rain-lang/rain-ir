/*!
Miscellaneous utilities and data structures used throughout the `rain` compiler
*/

#[cfg(feature = "symbol_table")]
pub mod symbol_table;

pub mod hash_cache;

/// Quickly implement `Display` using a given function
#[macro_export]
macro_rules! quick_display {
    ($t:ty, $s:ident, $fmt:ident => $e:expr) => {
        impl std::fmt::Display for $t {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                let $s = self;
                let $fmt = fmt;
                $e
            }
        }
    };
}

/// Implement `Debug` for a type which implements `Display`
#[macro_export]
macro_rules! debug_from_display {
    ($t:ty) => {
        impl std::fmt::Debug for $t {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                std::fmt::Display::fmt(self, fmt)
            }
        }
    };
}

/// Implement `Display` for a type using prettyprinting if it is enabled, and otherwise using a default function
#[macro_export]
macro_rules! display_pretty {
    ($t:ty, $fmt:ident, $s:ident => $default:expr) => {
        impl std::fmt::Display for $t {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                #[cfg(feature = "prettyprinter")]
                {
                    let mut printer = $crate::prettyprinter::PrettyPrinter::default();
                    $crate::prettyprinter::PrettyPrint::prettyprint(self, &mut printer, fmt)
                }
                #[cfg(not(feature = "prettyprinter"))]
                {
                    let $fmt = fmt;
                    let $s = self;
                    $default
                }
            }
        }
    };
    ($t:ty, $fmt_string:literal $(, $e:expr)*) => {
        display_pretty!($t, fmt, s => write!(fmt, $fmt_string $(, $e)*));
    };
}

/// Implement `From<T> for E` and implement `TryFrom<E> for T>` where `T` is an enum variant of `E`
#[macro_export]
macro_rules! enum_convert {
    (impl From<$T:ty> for $E:ty { as $f:expr, }) => {
        impl From<$T> for $E {
            fn from(v: $T) -> $E { $f(v) }
        }
    };
    (impl From<$T:ident> for $E:ident {}) => {
        enum_convert!(
            impl From<$T> for $E { as $E::$T, }
        );
    };
    (impl TryFrom<$E:ident> for $T:ty { $(match $ps:path,)+ $(try match $ts:path,)* $($from:tt => $to:tt,)* }) => {
        impl std::convert::TryFrom<$E> for $T {
            type Error = $E;
            fn try_from(v: $E) -> Result<$T, $E> {
                #[allow(unreachable_patterns)]
                match v.into() {
                    $($ps(v) => Ok(v.into()),)*
                    $($ts(v) => v.try_into().map_err(|err| err.into()),)*
                    $($from => $to,)*
                    e => Err(e.into())
                }
            }
        }
    };
    (impl TryFrom<$E:ident> for $T:ident {}) => {
        enum_convert!(
            impl TryFrom<$E> for $T { match $E::$T, }
        );
    };
    (impl Injection<$E:ident> for $T:ident { $(as $f:expr,)* $(match $ps:path,)* $(try match $ts:path,)* }) => {
            enum_convert!(impl From<$T> for $E { $(as $f,)* });
            enum_convert!(impl TryFrom<$E> for $T { $(match $ps,)* $(try match $ts:path,)* });
    };
    ($(impl $Tr:ident<$E:ident> for $T:ident { $(as $f:expr,)* $(match $ps:path,)* $(try match $ts:path,)* })*) => {
        $(enum_convert!(impl $Tr<$E> for $T { $(as $f,)* $(match $ps,)* $(try match $ts:path,)* });)*
    }
}