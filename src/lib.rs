//! Rust bindings for [libffi](https://sourceware.org/libffi/).

extern crate libc;

/// Unwrapped definitions imported from the C library (via bindgen).
pub mod raw;

/// A low-level wrapping of libffi. This layer makes no attempts at safety,
/// but tries to provide a somewhat more idiomatic interface. It also
/// re-exports types and constants necessary for using the library, so
/// it should not be generally necessary to use the `raw` module.
pub mod low;

/// Middle layer providing a somewhat safer (but still quite unsafe) API.
pub mod middle;

/// Representations of C types and arrays of thereof.
pub mod types;
