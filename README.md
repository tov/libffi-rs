# libffi-rs: Rust bindings for [libffi](https://sourceware.org/libffi/)

[![Build Status](https://travis-ci.org/tov/libffi-rs.svg?branch=master)](https://travis-ci.org/tov/libffi-rs)
[![Crates.io](https://img.shields.io/crates/v/libffi.svg?maxAge=2592000)](https://crates.io/crates/libffi)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](LICENSE-APACHE)

The C libffi library provides two main facilities: assembling calls
to functions dynamically, and creating closures that can be called
as ordinary C functions. In Rust, the latter means that we can turn
a Rust lambda (or any object implementing `Fn`/`FnMut`) into an
ordinary C function pointer that we can pass as a callback to C.

The easiest way to use this library is via the `high` layer module, but
more flexibility (and less checking) is provided by the `middle` and
`low` layers.

## Usage

It’s [on crates.io](https://crates.io/crates/libffi), so it can be
used by adding `libffi` to the dependencies in your project’s
`Cargo.toml`:

```toml
[dependencies]
libffi = "0.2"
```

It is necessary to have C [libffi](https://sourceware.org/libffi/)
installed first. (We’ve tested with libffi
version 3.2.1.)

## Example

In this example, we convert a Rust lambda containing a free variable
into an ordinary C code pointer. The type of `fun` below is
`extern "C" fn(u64, u64) -> u64`.

```rust
use libffi::high::Closure2;

let x = 5u64;
let f = |y: u64, z: u64| x + y + z;

let closure = Closure2::new(&f);
let fun     = closure.code_ptr();

assert_eq!(18, fun(6, 7));
```
