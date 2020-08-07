# rain

[![Pipeline status](https://gitlab.com/rain-lang/rain-ir/badges/master/pipeline.svg)](https://gitlab.com/rain-lang/rain-ir)
[![codecov](https://codecov.io/gl/rain-lang/rain-ir/branch/master/graph/badge.svg)](https://codecov.io/gl/rain-lang/rain-ir)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Gitpod ready-to-code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://gitlab.com/rain-lang/rain-ir)

`rain` is an implementation of an [RVSDG](https://arxiv.org/abs/1912.05036) with a strong linear type system and a concept of lifetimes, inspired by (and implemented in) Rust. Our goal is to build

- A purely functional intermediate representation which represents low-level programming concepts through linear typing and lifetimes, allowing lowering to efficient assembly without use of a garbage collector. We eventually hope to support bare-metal programming, including on embedded systems without an MMU.
- A type system strong enough to represent complex mathematical proofs both relating to program correctness and abstract mathematics in general, taking inspiration from [Homotopy Type Theory](https://homotopytypetheory.org/).
- A clean interface to existing programming languages which smoothly inter-operates with the type system.
- A performant and intuitive API to construct `rain` IR
- A highly optimized and parallel compiler and interpreter for `rain` on the desktop, and an interpreter for `rain` on the Web.
- A compact binary representation of `rain` IR that can be quickly serialized and deserialized.

`rain`, however, is not a programming language but an _intermediate representation_, and hence does not aim to necessarily be easily human-readable or human-writable. For instance, we only plan to support very limited type inference, with most of this responsibility being dedicated to front-ends.

Contributions, ideas and collaboration proposals are welcome: please make an issue or e-mail jad.ghalayini@gtc.ox.ac.uk.
