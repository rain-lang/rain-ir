/*!
Miscellaneous utilities and data structures used throughout the `rain` compiler
*/

use ref_cast::RefCast;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use triomphe::{Arc, ArcBorrow};

#[cfg(feature = "symbol_table")]
pub mod symbol_table;

pub mod hash_cache;

/// An implementation of ByAddress which can only be constructed if a value of the type
/// parameter `V` can be constructed, which of course is impossible without `unsafe` if
/// `V` has no public constructors.
#[repr(transparent)]
#[derive(RefCast)]
pub struct PrivateByAddr<T, V> {
    /// The underlying `Deref` implementor of this `PrivateByAddr`
    pub addr: T,
    hide: std::marker::PhantomData<V>,
}

impl<T, V> PrivateByAddr<T, V>
where
    T: Deref,
{
    /// Make a new `PrivateByAddr` instance
    pub fn make(addr: T, _validator: V) -> PrivateByAddr<T, V> {
        PrivateByAddr {
            addr,
            hide: std::marker::PhantomData,
        }
    }
}

impl<T, V> Clone for PrivateByAddr<T, V>
where
    T: Clone,
{
    fn clone(&self) -> PrivateByAddr<T, V> {
        PrivateByAddr {
            addr: self.addr.clone(),
            hide: self.hide,
        }
    }
}

impl<T, V> Copy for PrivateByAddr<T, V> where T: Copy {}

impl<T, V> Display for PrivateByAddr<T, V>
where
    T: Display,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Display::fmt(&self.addr, fmt)
    }
}

impl<T, V> Debug for PrivateByAddr<T, V>
where
    T: Debug,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(&self.addr, fmt)
    }
}

impl<T, V> Deref for PrivateByAddr<T, V>
where
    T: Deref,
{
    type Target = T::Target;
    fn deref(&self) -> &T::Target {
        self.addr.deref()
    }
}

impl<T, V> DerefMut for PrivateByAddr<T, V>
where
    T: DerefMut,
{
    fn deref_mut(&mut self) -> &mut T::Target {
        self.addr.deref_mut()
    }
}

impl<T, V, U> AsRef<U> for PrivateByAddr<T, V>
where
    T: AsRef<U>,
{
    fn as_ref(&self) -> &U {
        self.addr.as_ref()
    }
}

impl<T, V, U> AsMut<U> for PrivateByAddr<T, V>
where
    T: AsMut<U>,
{
    fn as_mut(&mut self) -> &mut U {
        self.addr.as_mut()
    }
}

impl<T, V> Hash for PrivateByAddr<T, V>
where
    T: Deref,
{
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.addr.deref(), hasher)
    }
}

impl<T, S, V, U> PartialEq<PrivateByAddr<S, U>> for PrivateByAddr<T, V>
where
    T: Deref,
    S: Deref,
{
    fn eq(&self, other: &PrivateByAddr<S, U>) -> bool {
        let self_ptr = self.deref() as *const _ as *const u8;
        let other_ptr = other.deref() as *const _ as *const u8;
        self_ptr == other_ptr
    }
}

impl<T, V> Eq for PrivateByAddr<T, V> where T: Deref {}

impl<T, V> PrivateByAddr<Arc<T>, V> {
    /// Borrow this `Arc` as a `PrivateByAddr<ArcBorrow<T>, V>`
    #[inline]
    pub fn borrow_arc(&self) -> PrivateByAddr<ArcBorrow<T>, V> {
        PrivateByAddr {
            addr: self.addr.borrow_arc(),
            hide: std::marker::PhantomData,
        }
    }
}

impl<'a, T, V> PrivateByAddr<ArcBorrow<'a, T>, V> {
    /// Clone this `Arc` as a `PrivateByAddr<Arc<T>, V>`, bumping the refcount
    #[inline]
    pub fn clone_arc(&self) -> PrivateByAddr<Arc<T>, V> {
        PrivateByAddr {
            addr: self.addr.clone_arc(),
            hide: std::marker::PhantomData,
        }
    }
    /// Get the referenced value with the lifetime of the `ArcBorrow`
    #[inline]
    pub fn get(&self) -> &'a T {
        self.addr.get()
    }
}

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
                    std::fmt::Display::fmt($crate::prettyprinter::PrettyPrint::prettyprintable(self), fmt)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Check the basic functionality of `PrivateByAddress`
    #[test]
    fn private_by_address() {
        let t_1 = PrivateByAddr::make(Arc::new(true), ());
        let t_2 = PrivateByAddr::make(Arc::new(true), ());
        assert_ne!(t_1, t_2);
        assert_eq!(t_1.deref(), t_2.deref());
        let t_1b = t_1.borrow_arc();
        assert_eq!(t_1b, t_1);
    }
}
