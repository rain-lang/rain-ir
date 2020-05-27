/*!
[![Documentation](https://docs.rs/rain-lang/badge.svg)](https://docs.rs/rain-lang/)
[![crates.io](https://img.shields.io/crates/v/rain-lang.svg)](https://crates.io/crates/rain-lang)
[![Downloads](https://img.shields.io/crates/d/rain-lang.svg)](https://crates.io/crates/rain-lang)
[![Pipeline status](https://gitlab.com/tekne/rain/badges/master/pipeline.svg)](https://gitlab.com/tekne/rain)
[![codecov](https://codecov.io/gl/tekne/rain/branch/master/graph/badge.svg)](https://codecov.io/gl/tekne/rain)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

`rain` is an implementation of an [RVSDG](https://arxiv.org/abs/1912.05036) with a concept of lifetimes,
inspired by (and implemented in) Rust. The goal is to build a purely functional, low-level intermediate
representation with a strong, linear type system incorporating some of the latest developments in compiler
design.

Contributions, ideas and collaboration proposals are welcome: please make an issue or e-mail jad.ghalayini@mail.utoronto.ca.
*/
#![forbid(unsafe_code, missing_docs, missing_debug_implementations)]

pub mod util;
pub mod value;

#[cfg(feature = "parser")]
pub mod parser;

#[cfg(feature = "prettyprinter")]
pub mod prettyprinter;
