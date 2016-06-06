//! A low-level wrapping of libffi, this layer makes no attempts at safety,
//! but tries to provide a somewhat more idiomatic interface.
//!
//! This module also re-exports types and constants necessary for using the
//! library, so it should not be generally necessary to use the `raw` module.
//! While this is a bit “Rustier” that [`raw`](../raw/index.html), I’ve
//! avoided drastic renaming in favor of hewing close to the libffi API.
//! See [`middle`](../middle/index.html) for an easier-to-use approach.

use std::mem;
use std::os::raw::{c_void, c_uint};

use raw;

/// The two kinds of errors reported by libffi.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    /// Given a bad type representation.
    BadTypedef,
    /// Given a bad or unrecognized ABI.
    BadAbi,
}

/// The `Result` type specialized for libffi `Error`s.
pub type Result<T> = ::std::result::Result<T, Error>;

// Converts the raw status type to a `Result`.
fn status_to_result<R>(status: raw::ffi_status, good: R) -> Result<R> {
    use raw::ffi_status::*;
    match status {
        FFI_OK => Ok(good),
        FFI_BAD_TYPEDEF => Err(Error::BadTypedef),
        FFI_BAD_ABI => Err(Error::BadAbi),
    }
}

/// Wraps a function pointer of unknown type.
///
/// This is used to make the API a bit easier to understand, and as a
/// simple type lint. As a `repr(C)` struct of one element, it should
/// be safe to transmute between `CodePtr` and `*mut c_void`.
#[derive(Clone, Copy, Debug, Hash)]
#[repr(C)]
pub struct CodePtr(pub *mut c_void);

// How useful is this type? Does it need all the methods?
impl CodePtr {
    /// Initializes a code pointer from a function pointer.
    ///
    /// This is useful mainly for talking to C APIs that take untyped
    /// callbacks specified in the API as having type `void(*)()`.
    pub fn from_fun(fun: unsafe extern "C" fn()) -> Self {
        CodePtr(fun as *mut c_void)
    }

    /// Initializes a code pointer from a void pointer.
    ///
    /// This is the other common type used in APIs (or at least in
    /// libffi) for untyped callback arguments.
    pub fn from_ptr(fun: *const c_void) -> Self {
        CodePtr(fun as *mut c_void)
    }

    /// Gets the code pointer typed as a `const void*`.
    ///
    /// This is the other common type used in APIs (or at least in
    /// libffi) for untyped callback arguments.
    pub fn as_ptr(&self) -> *const c_void {
        self.0
    }

    /// Gets the code pointer typed as a `void*`.
    ///
    /// This is the other common type used in APIs (or at least in
    /// libffi) for untyped callback arguments.
    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.0
    }

    /// Gets the code pointer typed as a C function pointer.
    ///
    /// This is useful mainly for talking to C APIs that take untyped
    /// callbacks specified in the API as having type `void(*)()`.
    ///
    /// # Safety
    ///
    /// There is no checking that the returned type reflects the actual
    /// parameter and return types of the function. Unless the C
    /// function actually has type `void(*)()`, it will need to be
    /// cast before it is called.
    pub fn as_fun(&self) -> &unsafe extern "C" fn() {
        unsafe {
            mem::transmute::<&*mut c_void, &unsafe extern "C" fn()>(&self.0)
        }
    }

    /// Gets the code pointer typed as a “safe” C function pointer.
    ///
    /// This is useful mainly for talking to C APIs that take untyped
    /// callbacks specified in the API as having type `void(*)()`.
    ///
    /// # Safety
    ///
    /// There isn’t necessarily anything actually safe about the resulting
    /// function pointer—it’s up to the caller to know what they’re
    /// doing within the unsafety boundary, or undefined behavior may
    /// result. In particular,
    /// there is no checking that the returned type reflects the actual
    /// parameter and return types of the function. Unless the C
    /// function actually has type `void(*)()`, it will need to be
    /// cast before it is called.
    pub unsafe fn as_safe_fun(&self) -> &extern "C" fn() {
        mem::transmute::<&*mut c_void, &extern "C" fn()>(&self.0)
    }
}

pub use raw::{ffi_abi, FFI_DEFAULT_ABI, _ffi_type as ffi_type, ffi_status,
              ffi_cif, ffi_closure};

/// Re-exports the `ffi_type` objects used to describe the types of
/// arguments and results.
///
/// These are from [`raw`](../../raw/index.html), but are renamed by
/// removing the `ffi_type_` prefix. For example, `raw::ffi_type_void`
/// becomes `low::types::void`.
pub mod types {
    pub use raw::{ffi_type_void as void,
                  ffi_type_uint8 as uint8,
                  ffi_type_sint8 as sint8,
                  ffi_type_uint16 as uint16,
                  ffi_type_sint16 as sint16,
                  ffi_type_uint32 as uint32,
                  ffi_type_sint32 as sint32,
                  ffi_type_uint64 as uint64,
                  ffi_type_sint64 as sint64,
                  ffi_type_float as float,
                  ffi_type_double as double,
                  ffi_type_pointer as pointer,
                  ffi_type_longdouble as longdouble,
                  ffi_type_complex_float as complex_float,
                  ffi_type_complex_double as complex_double,
                  ffi_type_complex_longdouble as complex_longdouble};
}

/// Type tags used in constructing and inspecting `ffi_type`s.
///
/// For atomic types this tag doesn’t matter because libffi predeclares
/// [an instance of each one](types/index.html). However, for composite
/// types (structs and complex numbers), we need to create a new
/// instance of the `ffi_type` struct. In particular, the `type_` field
/// contains a value that indicates what kind of type is represented,
/// and we use these values to indicate that that we are describing a
/// struct or complex type.
///
/// # Example
///
/// Suppose we have the following C struct:
///
/// ```c
/// struct my_struct {
///     uint16_t f1;
///     uint64_t f2;
/// };
/// ```
///
/// To pass it by value to a C function we can construct an
/// `ffi_type` as follows using `type_tag::STRUCT`:
///
/// ```
/// use std::ptr;
/// use libffi::low::{ffi_type, types, type_tag};
///
/// let mut elements = unsafe {
///     [ &mut types::uint16,
///       &mut types::uint64,
///       ptr::null::<ffi_type>() as *mut _ ]
/// };
///
/// let mut my_struct: ffi_type = Default::default();
/// my_struct.type_ = type_tag::STRUCT;
/// my_struct.elements = elements.as_mut_ptr();
/// ```
pub mod type_tag {
    use raw;
    use std::os::raw::c_ushort;

    /// Indicates a structure type.
    pub const STRUCT:  c_ushort = raw::ffi_type_enum::STRUCT as c_ushort;

    /// Indicates a complex number type.
    pub const COMPLEX: c_ushort = raw::ffi_type_enum::COMPLEX as c_ushort;
}

/// Initalizes a CIF (Call InterFace) with the given ABI and types.
///
/// Note that the CIF retains references to `rtype` and `atypes`, so if
/// they are no longer live when the CIF is used then the result is
/// undefined.
///
/// # Example
///
/// ```
/// use libffi::low::*;
///
/// let mut args: [*mut ffi_type; 2] = unsafe {
///     [ &mut types::sint32,
///       &mut types::uint64 ]
/// };
/// let mut cif: ffi_cif = Default::default();
///
/// unsafe {
///     prep_cif(&mut cif, FFI_DEFAULT_ABI, 2,
///              &mut types::pointer, args.as_mut_ptr())
/// }.unwrap();
/// ```
pub unsafe fn prep_cif(cif: *mut ffi_cif,
                       abi: ffi_abi,
                       nargs: usize,
                       rtype: *mut ffi_type,
                       atypes: *mut *mut ffi_type)
                       -> Result<()>
{
    let status = raw::ffi_prep_cif(cif, abi,
                                 nargs as c_uint,
                                 rtype, atypes);
    status_to_result(status, ())
}

/// Initalizes a CIF (Call InterFace) for a varargs function.
///
/// Note that the CIF retains references to `rtype` and `atypes`, so if
/// they are no longer live when the CIF is used then the result is
/// undefined.
pub unsafe fn prep_cif_var(cif: *mut ffi_cif,
                           abi: ffi_abi,
                           nfixedargs: usize,
                           ntotalargs: usize,
                           rtype: *mut ffi_type,
                           atypes: *mut *mut ffi_type)
                           -> Result<()>
{
    let status = raw::ffi_prep_cif_var(cif, abi,
                                     nfixedargs as c_uint,
                                     ntotalargs as c_uint,
                                     rtype, atypes);
    status_to_result(status, ())
}

/// Calls a C function as specified by a CIF.
///
/// C function `fun` is called with arguments `arg` using the using the
/// calling convention and types specified by `cif`.
///
/// # Example
///
/// ```
/// use std::os::raw::c_void;
/// use libffi::low::*;
///
/// extern "C" fn c_function(a: u64, b: u64) -> u64 { a + b }
///
/// let result = unsafe {
///     let mut args: Vec<*mut ffi_type> = vec![ &mut types::uint64,
///                                              &mut types::uint64 ];
///     let mut cif: ffi_cif = Default::default();
///
///     prep_cif(&mut cif, FFI_DEFAULT_ABI, 2,
///              &mut types::uint64, args.as_mut_ptr()).unwrap();
///
///     call(&mut cif, CodePtr(c_function as _),
///          vec![ &mut 4u64 as *mut _ as *mut c_void,
///                &mut 5u64 as *mut _ as *mut c_void ].as_mut_ptr())
/// };
///
/// assert_eq!(9, result);
/// ```
pub unsafe fn call<R>(cif:  *mut ffi_cif,
                      fun:  CodePtr,
                      args: *mut *mut c_void) -> R
{
    let mut result: R = mem::uninitialized();
    raw::ffi_call(cif,
                  Some(*fun.as_safe_fun()),
                  &mut result as *mut R as *mut c_void,
                  args);
    result
}

/// Allocates a closure.
///
/// Returns a pair of the writable closure object and the function
/// pointer for calling it. The latter lives until the former is freed
/// using [`closure_free`](../fn.closure_free.html).
///
/// # Example
///
/// ```
/// use libffi::low::*;
///
/// let (closure_handle, code_ptr) = closure_alloc();
/// ```
pub fn closure_alloc() -> (*mut ffi_closure, CodePtr) {
    unsafe {
        let mut code_pointer: *mut c_void = mem::uninitialized();
        let closure = raw::ffi_closure_alloc(mem::size_of::<ffi_closure>(),
                                             &mut code_pointer);
        (closure as *mut ffi_closure, CodePtr::from_ptr(code_pointer))
    }
}

/// Frees a closure.
///
/// # Example
///
/// ```
/// use libffi::low::*;
///
/// let (closure_handle, code_ptr) = closure_alloc();
///
/// // ...
///
/// unsafe {
///     closure_free(closure_handle);
/// }
/// ```
pub unsafe fn closure_free(closure: *mut ffi_closure) {
    raw::ffi_closure_free(closure as *mut c_void);
}

/// The type of function called by a closure.
///
/// `U` is the type of the user data captured by the closure and passed
/// to the callback, and `R` is the type of the result. The parameters
/// are not typed, since they are passed as a C array of `void*`.
pub type Callback<U, R>
    = unsafe extern "C" fn(cif:      &ffi_cif,
                           result:   &mut R,
                           args:     *const *const c_void,
                           userdata: &U);

/// The type of function called by a mutable closure.
///
/// `U` is the type of the user data captured by the closure and passed
/// to the callback, and `R` is the type of the result. The parameters
/// are not typed, since they are passed as a C array of `void*`.
pub type CallbackMut<U, R>
    = unsafe extern "C" fn(cif:      &ffi_cif,
                           result:   &mut R,
                           args:     *const *const c_void,
                           userdata: &mut U);

/// The callback type expected by `raw::ffi_prep_closure_loc`.
pub type RawCallback
    = unsafe extern "C" fn(cif:      *mut ffi_cif,
                           result:   *mut c_void,
                           args:     *mut *mut c_void,
                           userdata: *mut c_void);

/// Prepares a closure that calls the given callback function with the
/// given user data.
///
/// Note that the closure retains a reference to CIF `cif`, so that must
/// live as long as the closure refers to it or undefined behavior will
/// result.
///
/// # Example
///
/// ```
/// use libffi::low::*;
///
/// use std::mem;
/// use std::os::raw::c_void;
///
/// unsafe extern "C" fn callback(_cif: &ffi_cif,
///                               result: &mut u64,
///                               args: *const *const c_void,
///                               userdata: &u64)
/// {
///     let args: *const &u64 = mem::transmute(args);
///     *result = **args + *userdata;
/// }
///
/// fn twice(f: extern "C" fn(u64) -> u64, x: u64) -> u64 {
///     f(f(x))
/// }
///
/// unsafe {
///     let mut cif: ffi_cif = Default::default();
///     let mut args = [&mut types::uint64 as *mut _];
///     let mut userdata: u64 = 5;
///
///     prep_cif(&mut cif, FFI_DEFAULT_ABI, 1, &mut types::uint64,
///              args.as_mut_ptr()).unwrap();
///
///     let (closure, code) = closure_alloc();
///     let add5: extern "C" fn(u64) -> u64 = mem::transmute(code);
///
///     prep_closure(closure,
///                  &mut cif,
///                  callback,
///                  &mut userdata,
///                  CodePtr(add5 as _)).unwrap();
///
///     assert_eq!(11, add5(6));
///     assert_eq!(12, add5(7));
///
///     assert_eq!(22, twice(add5, 12));
/// }
/// ```
pub unsafe fn prep_closure<U, R>(closure:  *mut ffi_closure,
                                 cif:      *mut ffi_cif,
                                 callback: Callback<U, R>,
                                 userdata: *const U,
                                 code:     CodePtr)
    -> Result<()>
{
    let status = raw::ffi_prep_closure_loc
        (closure,
         cif,
         Some(mem::transmute::<Callback<U, R>, RawCallback>(callback)),
         userdata as *mut c_void,
         code.as_mut_ptr());
    status_to_result(status, ())
}

/// Prepares a mutable closure to call the given callback function with
/// the given user data.
///
/// Note that the closure retains a reference to CIF `cif`, so that must
/// live as long as the closure refers to it or undefined behavior will
/// result.
///
/// # Example
///
/// ```
/// use libffi::low::*;
///
/// use std::mem;
/// use std::os::raw::c_void;
///
/// unsafe extern "C" fn callback(_cif: &ffi_cif,
///                               result: &mut u64,
///                               args: *const *const c_void,
///                               userdata: &mut u64)
/// {
///     let args: *const &u64 = mem::transmute(args);
///     *result = *userdata;
///     *userdata += **args;
/// }
///
/// fn twice(f: extern "C" fn(u64) -> u64, x: u64) -> u64 {
///     f(f(x))
/// }
///
/// unsafe {
///     let mut cif: ffi_cif = Default::default();
///     let mut args = [&mut types::uint64 as *mut _];
///     let mut userdata: u64 = 5;
///
///     prep_cif(&mut cif, FFI_DEFAULT_ABI, 1, &mut types::uint64,
///              args.as_mut_ptr()).unwrap();
///
///     let (closure, code) = closure_alloc();
///     let add5: extern "C" fn(u64) -> u64 = mem::transmute(code);
///
///     prep_closure_mut(closure,
///                      &mut cif,
///                      callback,
///                      &mut userdata,
///                      CodePtr(add5 as _)).unwrap();
///
///     assert_eq!(5, add5(6));
///     assert_eq!(11, add5(7));
///
///     assert_eq!(19, twice(add5, 1));
/// }
/// ```
pub unsafe fn prep_closure_mut<U, R>(closure:  *mut ffi_closure,
                                     cif:      *mut ffi_cif,
                                     callback: CallbackMut<U, R>,
                                     userdata: *mut U,
                                     code:     CodePtr)
    -> Result<()>
{
    let status = raw::ffi_prep_closure_loc
        (closure,
         cif,
         Some(mem::transmute::<CallbackMut<U, R>, RawCallback>(callback)),
         userdata as *mut c_void,
         code.as_mut_ptr());
    status_to_result(status, ())
}
