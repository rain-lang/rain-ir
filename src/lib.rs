/*!
[![Pipeline status](https://gitlab.com/rain-lang/rain-ir/badges/master/pipeline.svg)](https://gitlab.com/rain-lang/rain-ir)
[![codecov](https://codecov.io/gl/tekne/rain/branch/master/graph/badge.svg)](https://codecov.io/gl/rain-lang/rain-ir)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Gitpod ready-to-code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://gitlab.com/rain-lang/rain-ir)

`rain` is an implementation of an [RVSDG](https://arxiv.org/abs/1912.05036) with a concept of lifetimes,
inspired by (and implemented in) Rust. The goal is to build a purely functional, low-level intermediate
representation with a strong, linear type system incorporating some of the latest developments in compiler
design.

Contributions, ideas and collaboration proposals are welcome: please make an issue or e-mail jad.ghalayini@mail.utoronto.ca.
*/
#![forbid(missing_docs, missing_debug_implementations)]
#![recursion_limit = "256"]
#[warn(clippy::all)]

pub mod data;
pub mod eval;
pub mod function;
pub mod graph;
pub mod lifetime;
pub mod primitive;
pub mod region;
pub mod typing;
pub mod util;
pub mod value;
pub mod control;

pub use rain_ast::tokens;

#[cfg(feature = "prettyprinter")]
pub mod prettyprinter;
