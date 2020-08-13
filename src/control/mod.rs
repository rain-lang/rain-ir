/*!
Control flow primitives

# Overview
This module contains the following primitive elements for describing control flow in `rain` programs
- [`ternary`](ternary) nodes for control flow on binary sum types such as booleans, bits, and binary sums of the form `A + B`
- [`switch`](switch) nodes for control flow on finite types
- [`rec`](rec) nodes for control flow on `n`-ary sum types and primitive recursion
- [`phi`](phi) nodes for arbitrary recursion, using the types in the `termination` module

The [`termination`](termination) module describes a type system for encapsulating non-termination without introducing inconsistencies, and the
[`nondeterministic`](nondeterministic) module describes a similar monadic type system for encapsulating non-parametric nondeterminism, as well as
nondeterministic control flow primitives.
*/

pub mod nondeterministic;
pub mod phi;
pub mod rec;
pub mod switch;
pub mod termination;
pub mod ternary;
