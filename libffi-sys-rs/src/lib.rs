#![doc(html_root_url = "https://docs.rs/libffi-sys/0.9.1")]
//! Low-level Rust bindings for [libffi](https://sourceware.org/libffi/)
//!
//! The C libffi library provides two main facilities: assembling calls
//! to functions dynamically, and creating closures that can be called
//! as ordinary C functions. This is an undocumented wrapper, generated
//! by bindgen, intended as the basis for higher-level bindings, but you
//! can see the [C libffi
//! documentation](http://www.atmark-techno.com/~yashi/libffi.html).
//!
//! See [the libffi crate](https://crates.io/crates/libffi/) for a
//! higher-level API.
//!
//! # Usage
//!
//! `libffi-sys` can either build its own copy of the libffi C library [from
//! github](https://github.com/libffi/libffi) or it can link against your
//! system’s C libffi. By default it builds its own because many systems
//! ship with an old C libffi; this requires that you have a working make,
//! C compiler, automake, and autoconf first. If your system libffi
//! is new enough (v3.2.1 as of October 2019), you can instead enable the
//! `system` feature flag to use that. If you want this crate to build
//! a C libffi for you, add
//!
//! ```toml
//! [dependencies]
//! libffi-sys = "0.9.1"
//! ```
//!
//! to your `Cargo.toml`. If you want to use your system C libffi, then
//!
//! ```toml
//! [dependencies.libffi-sys]
//! version = "0.9.1"
//! features = ["system"]
//! ```
//!
//! to your `Cargo.toml` instead.
//!
//! This crate supports Rust version 1.32 and later.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/generated.rs"));
