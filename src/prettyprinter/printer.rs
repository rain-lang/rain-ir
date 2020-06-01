/*
The actual, conditionally compiled prettyprinter implementation
*/

use crate::util::symbol_table::SymbolTable;
use crate::value::{typing::{Type, Typed}, ValRef, Value, NormalValue};
use crate::{debug_from_display, quick_display};
use ahash::RandomState;
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
quick_display!(VirtualRegister, s, fmt => write!(fmt, "%{}", s));

/// A prettyprinter for `rain` values
#[derive(Clone)]
pub struct PrettyPrinter<I = VirtualRegister, S: BuildHasher = RandomState> {
    symbols: SymbolTable<*const NormalValue, I, S>,
    unique: usize,
    scope: usize,
    max_tabs: u16
}

impl<I: Debug, S: BuildHasher> Debug for PrettyPrinter<I, S> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.debug_struct("PrettyPrinter")
            .field("symbols", &self.symbols)
            .field("unique", &self.unique)
            .field("scope", &self.scope)
            .field("max_tabs", &self.max_tabs)
            .finish()
    }
}

/// The size of prettyprinter stack to use before allocating
const PRETTYPRINTER_STACK_DEPTH: usize = 16;

/// The default maximum number of tags for a prettyprinter
pub const DEFAULT_MAX_TABS: u16 = 4;

impl<I: Display + From<usize> + Sized> PrettyPrinter<I> {
    /// Create a new prettyprinter
    pub fn new() -> PrettyPrinter<I> {
        PrettyPrinter {
            symbols: SymbolTable::new(),
            unique: 0,
            scope: 0,
            max_tabs: DEFAULT_MAX_TABS
        }
    }
    /// Print the appropriate number of tabs for the given scope level, up to the maximum
    pub fn print_tabs(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        let to_print = self.scope.min(self.max_tabs as usize);
        for _ in 0..to_print {
            write!(fmt, "\t")?;
        }
        Ok(())
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
        if self.symbols.contains_key(&(value.deref() as *const NormalValue)) {
            return Ok(0);
        }
        visit_stack.push((value, 0));
        while let Some((top, mut ix)) = visit_stack.pop() {
            while ix < top.no_deps() {
                let dep = top.as_norm().get_dep(ix);
                if self.symbols.contains_key(&(dep.deref() as *const NormalValue)) {
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
                if !ty.is_universe() { // Print the dependencies of non-universe types
                    visit_stack.push((top, ix));
                    visit_stack.push((ty.as_val(), 0));
                    continue;
                }
            }
            if ix > top.no_deps() {
                // Print the dependency, creating a new name
                let name: I = self.unique.into();
                let ty = top.ty();
                // Print the correct number of tabs (corresponding to the current scope level)
                self.print_tabs(fmt)?;
                if !ty.is_universe() { // Only print the type of non-types
                    write!(fmt, "#let {}: {} = ", name, ty)?;
                } else {
                    write!(fmt, "#let {} = ", name)?;
                }
                top.prettyprint(self, fmt)?;
                writeln!(fmt, ";")?;
                self.symbols.def(top.deref(), name);
                // Record the increase in the number of defined names
                self.unique += 1;
                new_deps += 1;
                // We're done with this iteration: pop again
                continue;
            }
        }
        Ok(new_deps)
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
