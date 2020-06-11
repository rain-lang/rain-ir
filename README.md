[![Gitpod ready-to-code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://gitlab.com/tekne/rain)

# rain
[![Documentation](https://docs.rs/rain-lang/badge.svg)](https://docs.rs/rain-lang/)
[![crates.io](https://img.shields.io/crates/v/rain-lang.svg)](https://crates.io/crates/rain-lang)
[![Downloads](https://img.shields.io/crates/d/rain-lang.svg)](https://crates.io/crates/rain-lang)
[![Pipeline status](https://gitlab.com/tekne/rain/badges/master/pipeline.svg)](https://gitlab.com/tekne/rain)
[![codecov](https://codecov.io/gl/tekne/rain/branch/master/graph/badge.svg)](https://codecov.io/gl/tekne/rain)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

`rain` is an implementation of an [RVSDG](https://arxiv.org/abs/1912.05036) with a strong linear type system and a concept of lifetimes, inspired by (and implemented in) Rust. Our goal is to build
- A purely functional intermediate representation which represents low-level programming concepts through linear typing and lifetimes, allowing lowering to efficient assembly without use of a garbage collector. We eventually hope to support bare-metal programming, including on embedded systems without an MMU.
- A type system strong enough to represent complex mathematical proofs both relating to program correctness and abstract mathematics in general, taking inspiration from [Homotopy Type Theory](https://homotopytypetheory.org/).
- A clean interface to existing programming languages which smoothly inter-operates with the type system.
- A performant and intuitive API to construct `rain` IR
- A highly optimized and parallel compiler and interpreter for `rain` on the desktop, and an interpreter for `rain` on the Web.
- A compact binary representation of `rain` IR that can be quickly serialized and deserialized.

`rain`, however, is not a programming language but an *intermediate representation*, and hence does not aim to necessarily be easily human-readable or human-writable. For instance, we only plan to support very limited type inference, with most of this responsibility being dedicated to front-ends.

Contributions, ideas and collaboration proposals are welcome: please make an issue or e-mail jad.ghalayini@mail.utoronto.ca.
