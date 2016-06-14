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

It’s [on crates.io](https://crates.io/crates/libffi-sys), but before you
build it, make sure you have the dependencies installed first:

  - An up-to-date version of C [libffi](https://sourceware.org/libffi/)
    Version 3.2.1 is known to work. Earlier versions, such as the
    versions that come with Mac OS and Fedora, are known not to; neither
    will the version installed by Homebrew (3.0.13).

  - [`pkg-config`](https://www.freedesktop.org/wiki/Software/pkg-config/),
    which you probably already have if you’re on Linux. For Mac users,
    the version installed by Homebrew is up to date. (I don’t know how
    this works on Windows; contact me if you’d like to help figure it
    out.)

Then add

```toml
[dependencies]
libffi = "0.3"
```

to your `Cargo.toml` and

```rust
extern crate libffi;
```

to your crate root.
