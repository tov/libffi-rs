//! Simple dynamic calls.
//!
//! This API allows us to call a code pointer with an array of
//! arguments, using libffi to set up the call.
//!
//! # Examples
//!
//! ```
//! extern "C" fn hypot(x: f32, y: f32) -> f32 {
//!     (x * x + y * y).sqrt()
//! }
//!
//! use libffi::ffi_call;
//!
//! let result = unsafe { ffi_call!{ hypot(3f32, 4f32) -> f32 } };
//!
//! assert!((result - 5f32).abs() < 0.0001);
//! ```

use crate::middle;
pub use middle::CodePtr;

/// Encapsulates an argument with its type information.
///
/// In order to set up calls using [`fn@call`], we
/// need to wrap (a reference to) each argument in an `Arg`. The usual
/// way to do this is with function [`arg`].
#[derive(Clone, Debug)]
pub struct Arg<'a> {
    // There should be some type T such that type_ is the middle-layer
    // value of Type<T> and value is T::reify().
    type_: middle::Type,
    value: middle::Arg<'a>,
}

impl<'a> Arg<'a> {
    /// Wraps an argument reference for passing to [`fn@call`].
    ///
    /// For a shorter alias of the same, see [`fn@arg`].
    pub fn new<T: super::CType>(arg: &'a T) -> Self {
        Arg {
            type_: T::reify().into_middle(),
            value: middle::Arg::new(arg),
        }
    }
}

/// Constructs an [`Arg`] for passing to [`fn@call`].
pub fn arg<T: super::CType>(arg: &T) -> Arg {
    Arg::new(arg)
}

/// Performs a dynamic call to a C function.
///
/// To reduce boilerplate, see [`ffi_call!`].
///
/// # Examples
///
/// ```
/// extern "C" fn hypot(x: f32, y: f32) -> f32 {
///     (x * x + y * y).sqrt()
/// }
///
/// use libffi::high::call::*;
///
/// let result = unsafe {
///     call::<f32>(CodePtr(hypot as *mut _), &[arg(&3f32), arg(&4f32)])
/// };
///
/// assert!((result - 5f32).abs() < 0.0001);
/// ```
/// # Safety
/// The signature of the function pointer must match the types of the arguments and the return type.
/// If the types do not match, we get UB.
pub unsafe fn call<R: super::CRetType>(fun: CodePtr, args: &[Arg]) -> R {
    let types = args.iter().map(|arg| arg.type_.clone());
    let cif = middle::Cif::new(types, R::get_return_type());

    let values = args
        .iter()
        .map(|arg| arg.value.clone())
        .collect::<alloc::vec::Vec<_>>();
    cif.call::<R>(fun, &values)
}

/// Performs a dynamic call to a C function.
///
/// This macro provides sugar for [`high::arg`](crate::high::arg) and
/// [`high::call`](fn@crate::high::call). For more control, see
/// [`high::call`](fn@crate::high::call).
///
/// # Examples
///
/// ```
/// extern "C" fn hypot(x: f32, y: f32) -> f32 {
///     (x * x + y * y).sqrt()
/// }
///
/// use libffi::ffi_call;
///
/// let result = unsafe { ffi_call!{ hypot(3f32, 4f32) -> f32 } };
///
/// assert!((result - 5f32).abs() < 0.0001);
/// ```
#[macro_export]
macro_rules! ffi_call {

    { ( $fun:expr ) ( $( $arg:expr ),* ) -> $ty:ty }
    =>
    {
        $crate::high::call::call::<$ty>(
            $crate::high::call::CodePtr($fun as *mut _),
            &[$($crate::high::call::arg(&$arg)),*])
    };

    { $fun:ident ( $( $arg:expr ),* ) -> $ty:ty }
    =>
    { ffi_call!{ ($fun)($($arg),*) -> $ty } };

    { ( $fun:expr ) ( $( $arg:expr ),* ) }
    =>
    { ffi_call!{ ($fun)($(arg),*) -> () } };

    { $fun:ident ( $( $arg:expr ),* ) }
    =>
    { ffi_call!{ ($fun)($($arg),*) -> () } };

}
