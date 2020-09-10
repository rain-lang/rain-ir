/*!
Utilities to do with formatting, debug printing, prettyprinting, etc.
*/


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
            fn prettyprint<I>(
                &self,
                _printer: &mut $crate::prettyprinter::PrettyPrinter<I>,
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
                    std::fmt::Display::fmt($crate::prettyprinter::PrettyPrint::prp(self), fmt)
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