/*!
Miscellaneous utilities and data structures used throughout the `rain` compiler
*/

#[cfg(feature = "symbol_table")]
pub mod symbol_table;

pub mod hash_cache;

/// Quickly implement `Display` using a given function or format string
#[macro_export]
macro_rules! quick_display {
    ($t:ty, $s:pat, $fmt:pat => $e:expr) => {
        impl std::fmt::Display for $t {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                let $s = self;
                let $fmt = fmt;
                $e
            }
        }
    };
    ($t:ty, $fmt_string:literal $(, $e:expr)*) => {
        $crate::quick_display!($t, _, fmt => write!(fmt, $fmt_string $(, $e)*));
    };
}

/// Quickly implement `Display` or `PrettyPrint` using a given function or format string
#[macro_export]
macro_rules! quick_pretty {
    ($t:ty, $s:pat, $fmt:pat => $e:expr) => {
        $crate::quick_display!($t, $s, $fmt => $e);
        $crate::display_pretty!($t);
    };
    ($t:ty, $fmt_string:literal $(, $e:expr)*) => {
        $crate::quick_display!($t, $fmt_string $(,$e)*);
        $crate::display_pretty!($t);
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

/// Implement `PrettyPrint` for a type using `Display` if prettyprinting is enabled, otherwise do nothing
#[macro_export]
macro_rules! display_pretty {
    ($t:ty) => {
        #[cfg(feature = "prettyprinter")]
        impl $crate::prettyprinter::PrettyPrint for $t {
            fn prettyprint(
                &self,
                _printer: &mut $crate::prettyprinter::PrettyPrinter,
                fmt: &mut std::fmt::Formatter,
            ) -> Result<(), std::fmt::Error> {
                std::fmt::Display::fmt(self, fmt)
            }
        }
    };
}

/// Implement `Display` for a type using prettyprinting if it is enabled, and otherwise using a default function
#[macro_export]
macro_rules! pretty_display {
    ($t:ty, $s:pat, $fmt:pat => $default:expr) => {
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
        pretty_display!($t, _, fmt => write!(fmt, $fmt_string $(, $e)*));
    };
}

/// Implement `From<T> for E` and implement `TryFrom<E> for T>` where `T` is an enum variant of `E`
#[macro_export]
macro_rules! enum_convert {
    (impl From<$T:ident> for $E:ident { as $En:ident::$V:ident, }) => {
        impl From<$T> for $E {
            fn from(v: $T) -> $E { $En::$V(v).into() }
        }
    };
    (impl From<$T:ident> for $E:ident { as $En:ident, }) => {
        enum_convert!(
            impl From<$T> for $E { as $En::$T, }
        );
    };
    (impl From<$T:ident> for $E:ident {}) => {
        enum_convert!(
            impl From<$T> for $E { as $E, }
        );
    };
    (impl TryFrom<$E:ident> for $T:ident { as $V:ident, $($from:tt => $to:tt,)* }) => {
        impl std::convert::TryFrom<$E> for $T {
            type Error = $E;
            fn try_from(v: $E) -> Result<$T, $E> {
                #[allow(unreachable_patterns)]
                let v: $V = v.into();
                match v {
                    $V::$T(v) => Ok(v.into()),
                    $($from => $to,)*
                    e => Err(e.into())
                }
            }
        }
    };
    (impl TryFrom<$E:ident> for $T:ident {}) => {
        enum_convert!(
            impl TryFrom<$E> for $T { as $E, }
        );
    };
    (impl Injection<$E:ident> for $T:ident { $(as $f:expr,)* $(match $ps:path,)* $(try match $ts:path,)* }) => {
            enum_convert!(impl From<$T> for $E { $(as $f,)* });
            enum_convert!(impl TryFrom<$E> for $T { $(match $ps,)* $(try match $ts:path,)* });
    };
    ($(impl $Tr:ident<$E:ident> for $T:ident { $(as $t:ident,)* $($from:tt => $to:tt,)* } )*) => {
        $(
            enum_convert!{
                impl $Tr<$E> for $T { $(as $t,)* $($from:tt => $to:tt,)* }
            }
        )*
    }
}
