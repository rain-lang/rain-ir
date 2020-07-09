/*!
A prettyprinter for `rain` programs
*/
use crate::tokens::*;
use crate::typing::{Type, Typed};
use crate::value::{NormalValue, ValRef, Value};
use crate::{debug_from_display, quick_display};
use fxhash::FxBuildHasher;
use hayami::SymbolTable;
use ref_cast::RefCast;
use smallvec::SmallVec;
use std::default::Default;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::BuildHasher;
use std::ops::Deref;

/// The virtual register name format for `rain` values
#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct VirtualRegister(pub usize);

impl From<usize> for VirtualRegister {
    fn from(u: usize) -> VirtualRegister {
        VirtualRegister(u)
    }
}

debug_from_display!(VirtualRegister);
quick_display!(VirtualRegister, s, fmt => write!(fmt, "%{}", s.0));

/// A prettyprinter for `rain` values
#[derive(Clone)]
pub struct PrettyPrinter<I = VirtualRegister, S: BuildHasher = FxBuildHasher> {
    symbols: SymbolTable<*const NormalValue, I, S>,
    unique: usize,
    scope: Vec<bool>,
    open_scopes: usize,
    max_tabs: u16,
}

impl<I: Debug, S: BuildHasher> Debug for PrettyPrinter<I, S> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.debug_struct("PrettyPrinter")
            .field("symbols", &self.symbols)
            .field("unique", &self.unique)
            .field("scope", &self.scope)
            .field("open_scopes", &self.open_scopes)
            .field("max_tabs", &self.max_tabs)
            .finish()
    }
}

/// The size of prettyprinter stack to use before allocating
const PRETTYPRINTER_STACK_DEPTH: usize = 16;

/// The default maximum number of tags for a prettyprinter
pub const DEFAULT_MAX_TABS: u16 = 4;

/// Display a value using an empty prettyprinter
#[derive(Debug, Copy, Clone, RefCast, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct PrettyPrintable<T>(T);

impl<I: Display + From<usize> + Sized> PrettyPrinter<I> {
    /// Create a new prettyprinter
    pub fn new() -> PrettyPrinter<I> {
        PrettyPrinter {
            symbols: SymbolTable::new(),
            unique: 0,
            scope: Vec::new(),
            open_scopes: 0,
            max_tabs: DEFAULT_MAX_TABS,
        }
    }
    /// Print the appropriate number of tabs for the given scope level, up to the maximum
    pub fn print_tabs(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        let to_print = self.open_scopes.min(self.max_tabs as usize);
        for _ in 0..to_print {
            write!(fmt, "\t")?;
        }
        Ok(())
    }
    /// Check whether a `ValId` has an associated identifier
    pub fn has_id(&self, value: ValRef) -> bool {
        self.symbols
            .contains_key(&(value.deref() as *const NormalValue))
    }
    /// Try to prettyprint a `ValId`'s associated identifier. Return whether it was printed or not
    pub fn try_prettyprint(&self, fmt: &mut Formatter, value: ValRef) -> Result<bool, fmt::Error> {
        if let Some(id) = self.symbols.get(&(value.deref() as *const NormalValue)) {
            write!(fmt, "{}", id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    /// Prettyprint a `ValId` and its dependencies as `let` statements, avoiding recursion.
    /// Return the number of new definitions, if any.
    ///
    /// This is a depth-first search, so we should never see the same dependency twice.
    pub fn prettyprint_valid_and_deps(
        &mut self,
        fmt: &mut Formatter,
        value: ValRef,
    ) -> Result<usize, fmt::Error> {
        let mut new_deps = 0;
        let mut visit_stack = SmallVec::<[(ValRef, usize); PRETTYPRINTER_STACK_DEPTH]>::new();
        if self.has_id(value) {
            return Ok(0);
        }
        visit_stack.push((value, 0));
        while let Some((top, mut ix)) = visit_stack.pop() {
            while ix < top.no_deps() {
                let dep = top.as_norm().get_dep(ix);
                if self.has_id(dep.borrow_val()) && dep.no_deps() > 0 {
                    // Note we avoid printing dependencies with no dependencies as `let` statements
                    ix += 1;
                } else {
                    // Push the new dependency, and the old dependency
                    visit_stack.push((top, ix + 1));
                    visit_stack.push((dep.borrow_val(), 0));
                    continue;
                }
            }
            if ix == top.no_deps() {
                ix += 1;
                let ty = top.as_norm().ty();
                if !ty.is_universe() {
                    // Print the dependencies of non-universe types
                    visit_stack.push((top, ix));
                    visit_stack.push((ty.as_val(), 0));
                    continue;
                }
            }
            if ix > top.no_deps() {
                // Print the dependency, creating a new name
                let name: I = self.unique.into();
                let ty = top.ty();
                // If the current scope is not open, open it. If not in a scope, ignore.
                if let Some(top) = self.scope.last() {
                    // Immutable printing
                    if !top {
                        self.print_tabs(fmt)?;
                        writeln!(fmt, "{{")?;
                    }
                }
                if let Some(top) = self.scope.last_mut() {
                    // Mutable editing
                    if !(*top) {
                        *top = true;
                        self.open_scopes += 1;
                    }
                }
                // Print the correct number of tabs (corresponding to the current scope level)
                self.print_tabs(fmt)?;
                if !ty.is_universe() {
                    // Only print the type of non-types
                    write!(
                        fmt,
                        "{} {}{} {} {} ",
                        KEYWORD_LET, name, JUDGE_TYPE, ty, ASSIGN
                    )?;
                } else {
                    write!(fmt, "{} {} {} ", KEYWORD_LET, name, ASSIGN)?;
                }
                top.prettyprint(self, fmt)?;
                writeln!(fmt, "{}", STATEMENT_DELIM)?;
                self.symbols.insert(top.deref(), name);
                // Record the increase in the number of defined names
                self.unique += 1;
                new_deps += 1;
                // We're done with this iteration: pop again
                continue;
            }
        }
        Ok(new_deps)
    }
    /// Print and register a new index for a value, along with it's type
    pub fn prettyprint_index(
        &mut self,
        fmt: &mut Formatter,
        value: ValRef,
    ) -> Result<(), fmt::Error> {
        let name: I = self.unique.into();
        write!(fmt, "{}: {}", name, value.ty())?;
        self.symbols.insert(value.deref(), name);
        self.unique += 1;
        Ok(())
    }
    /// Prettyprint a value's dependencies as `let` statements, if not already printed.
    pub fn prettyprint_deps<V: Value>(
        &mut self,
        fmt: &mut Formatter,
        value: &V,
    ) -> Result<usize, fmt::Error> {
        let mut new_deps = 0;
        for dep in value.deps().iter() {
            new_deps += self.prettyprint_valid_and_deps(fmt, dep.borrow_val())?
        }
        Ok(new_deps)
    }
    /// Push a new scope
    pub fn push_scope(&mut self) {
        self.scope.push(false)
    }
    /// Pop a scope, closing it if necessary. If a scope was at the top, return whether it was open.
    pub fn pop_scope(&mut self, fmt: &mut Formatter) -> Result<Option<bool>, fmt::Error> {
        if let Some(top) = self.scope.pop() {
            if top {
                self.open_scopes -= 1;
                self.print_tabs(fmt)?;
                writeln!(fmt, "\n}}")?;
            }
            Ok(Some(top))
        } else {
            Ok(None)
        }
    }
    /// Prettyprint a value in a new scope
    pub fn scoped_print<V: Value + PrettyPrint>(
        &mut self,
        fmt: &mut Formatter,
        value: &V,
    ) -> Result<usize, fmt::Error> {
        self.push_scope();
        let vals = self.prettyprint_deps(fmt, value)?;
        value.prettyprint(self, fmt)?;
        self.pop_scope(fmt)?;
        Ok(vals)
    }
    /// Lookup a value in this symbol table
    pub fn lookup(&self, value: &NormalValue) -> Option<&I> {
        self.symbols.get(&(value as *const NormalValue))
    }
}

impl Default for PrettyPrinter {
    fn default() -> PrettyPrinter {
        Self::new()
    }
}

/// A value which can be prettyprinted
pub trait PrettyPrint {
    /// Prettyprint a value using a given printer
    fn prettyprint<I: From<usize> + Display>(
        &self,
        printer: &mut PrettyPrinter<I>,
        fmt: &mut Formatter,
    ) -> Result<(), fmt::Error>;
    /// Create a pretty-printable object from this one
    fn prp(&self) -> &PrettyPrintable<Self>
    where
        Self: Sized,
    {
        RefCast::ref_cast(self)
    }
}

impl<T: PrettyPrint> Display for PrettyPrintable<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        let mut printer = PrettyPrinter::default();
        self.0.prettyprint(&mut printer, fmt)
    }
}

impl<T: PrettyPrint> PrettyPrint for PrettyPrintable<T> {
    fn prettyprint<I: From<usize> + Display>(
        &self,
        printer: &mut PrettyPrinter<I>,
        fmt: &mut Formatter,
    ) -> Result<(), fmt::Error> {
        self.0.prettyprint(printer, fmt)
    }
}

/// Implement `PrettyPrint` using `Display`
#[macro_export]
macro_rules! prettyprint_by_display {
    ($t:ty) => {
        impl $crate::prettyprinter::PrettyPrint for $t {
            fn prettyprint<I: From<usize> + Display>(
                &self,
                printer: &mut $crate::prettyprinter::PrettyPrinter<I>,
                fmt: &mut std::fmt::Formatter,
            ) -> Result<(), std::fmt::Error> {
            }
        }
    };
}
