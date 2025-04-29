//! A low-level wrapping of libffi, this layer makes no attempts at safety,
//! but tries to provide a somewhat more idiomatic interface.
//!
//! This module also re-exports types and constants necessary for using the
//! library, so it should not be generally necessary to use the `raw` module.
//! While this is a bit “Rustier” than [`raw`](crate::raw), I’ve
//! avoided drastic renaming in favor of hewing close to the libffi API.
//! See [`middle`](crate::middle) for an easier-to-use approach.

use core::ffi::{c_uint, c_void};
use core::mem;

use crate::raw;

/// The two kinds of errors reported by libffi.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    /// Given a bad or unsupported type representation.
    Typedef,
    /// Given a bad or unsupported ABI.
    Abi,
}

/// The [`std::result::Result`] type specialized for libffi [`Error`]s.
pub type Result<T> = ::core::result::Result<T, Error>;

// Converts the raw status type to a `Result`.
fn status_to_result<R>(status: raw::ffi_status, good: R) -> Result<R> {
    if status == raw::ffi_status_FFI_OK {
        Ok(good)
    } else if status == raw::ffi_status_FFI_BAD_TYPEDEF {
        Err(Error::Typedef)
    }
    // If we don't recognize the status, that is an ABI error:
    else {
        Err(Error::Abi)
    }
}

/// Wraps a function pointer of unknown type.
///
/// This is used to make the API a bit easier to understand, and as a
/// simple type lint. As a `repr(C)` struct of one element, it should
/// be safe to transmute between `CodePtr` and `*mut c_void`, or between
/// collections thereof.
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
        unsafe { self.as_any_ref_() }
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
        self.as_any_ref_()
    }

    pub(crate) unsafe fn as_any_ref_<T>(&self) -> &T {
        &*(&self.0 as *const _ as *const T)
    }

    /// Gets the code pointer typed as a `const void*`.
    ///
    /// This is the other common type used in APIs (or at least in
    /// libffi) for untyped callback arguments.
    pub fn as_ptr(self) -> *const c_void {
        self.0
    }

    /// Gets the code pointer typed as a `void*`.
    ///
    /// This is the other common type used in APIs (or at least in
    /// libffi) for untyped callback arguments.
    pub fn as_mut_ptr(self) -> *mut c_void {
        self.0
    }
}

pub use raw::{
    ffi_abi, ffi_abi_FFI_DEFAULT_ABI, ffi_arg, ffi_cif, ffi_closure, ffi_sarg, ffi_status, ffi_type,
};

/// Re-exports the [`ffi_type`] objects used to describe the types of
/// arguments and results.
///
/// These are from [the raw layer](crate::raw), but are renamed by
/// removing the `ffi_type_` prefix. For example, [`raw::ffi_type_void`]
/// becomes [`types::void`].
pub mod types {
    pub use crate::raw::{
        ffi_type_double as double, ffi_type_float as float, ffi_type_pointer as pointer,
        ffi_type_sint16 as sint16, ffi_type_sint32 as sint32, ffi_type_sint64 as sint64,
        ffi_type_sint8 as sint8, ffi_type_uint16 as uint16, ffi_type_uint32 as uint32,
        ffi_type_uint64 as uint64, ffi_type_uint8 as uint8, ffi_type_void as void,
    };

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    pub use crate::raw::ffi_type_longdouble as longdouble;

    #[cfg(feature = "complex")]
    pub use crate::raw::{
        ffi_type_complex_double as complex_double, ffi_type_complex_float as complex_float,
    };

    #[cfg(feature = "complex")]
    #[cfg(not(all(target_arch = "arm")))]
    pub use crate::raw::ffi_type_complex_longdouble as complex_longdouble;
}

/// Type tags used in constructing and inspecting [`ffi_type`]s.
///
/// For atomic types this tag doesn’t matter because libffi predeclares
/// [an instance of each one](mod@types). However, for composite
/// types (structs and complex numbers), we need to create a new
/// instance of the [`ffi_type`] struct. In particular, the `type_` field
/// contains a value that indicates what kind of type is represented,
/// and we use these values to indicate that that we are describing a
/// struct or complex type.
///
/// # Examples
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
///       ptr::null_mut::<ffi_type>() ]
/// };
///
/// let mut my_struct: ffi_type = Default::default();
/// my_struct.type_ = type_tag::STRUCT;
/// my_struct.elements = elements.as_mut_ptr();
/// ```
pub mod type_tag {
    use crate::raw;
    use core::ffi::c_ushort;

    /// Indicates a structure type.
    pub const STRUCT: c_ushort = raw::ffi_type_enum_STRUCT as c_ushort;

    /// Indicates a complex number type.
    ///
    /// This item is enabled by `#[cfg(feature = "complex")]`.
    #[cfg(feature = "complex")]
    pub const COMPLEX: c_ushort = raw::ffi_type_enum_COMPLEX as c_ushort;
}

/// Initalizes a CIF (Call Interface) with the given ABI
/// and types.
///
/// We need to initialize a CIF before we can use it to call a function
/// or create a closure. This function lets us specify the calling
/// convention to use and the argument and result types. For varargs
/// CIF initialization, see [`prep_cif_var`].
///
///
/// # Safety
///
/// The CIF `cif` retains references to `rtype` and `atypes`, so if
/// they are no longer live when the CIF is used then the behavior is
/// undefined.
///
/// # Arguments
///
/// - `cif` — the CIF to initialize
/// - `abi` — the calling convention to use
/// - `nargs` — the number of arguments
/// - `rtype` — the result type
/// - `atypes` — the argument types (length must be at least `nargs`)
///
/// # Result
///
/// `Ok(())` for success or `Err(e)` for failure.
///
/// # Examples
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
///     prep_cif(&mut cif, ffi_abi_FFI_DEFAULT_ABI, 2,
///              &mut types::pointer, args.as_mut_ptr())
/// }.unwrap();
/// ```
pub unsafe fn prep_cif(
    cif: *mut ffi_cif,
    abi: ffi_abi,
    nargs: usize,
    rtype: *mut ffi_type,
    atypes: *mut *mut ffi_type,
) -> Result<()> {
    let status = raw::ffi_prep_cif(cif, abi, nargs as c_uint, rtype, atypes);
    status_to_result(status, ())
}

/// Initalizes a CIF (Call Interface) for a varargs function.
///
/// We need to initialize a CIF before we can use it to call a function
/// or create a closure. This function lets us specify the calling
/// convention to use and the argument and result types. For non-varargs
/// CIF initialization, see [`prep_cif`].
///
/// # Safety
///
/// The CIF `cif` retains references to `rtype` and `atypes`, so if
/// they are no longer live when the CIF is used then the behavior is
/// undefined.
///
/// # Arguments
///
/// - `cif` — the CIF to initialize
/// - `abi` — the calling convention to use
/// - `nfixedargs` — the number of fixed arguments
/// - `ntotalargs` — the total number of arguments, including fixed and
///    var args
/// - `rtype` — the result type
/// - `atypes` — the argument types (length must be at least `nargs`)
///
/// # Result
///
/// `Ok(())` for success or `Err(e)` for failure.
///
pub unsafe fn prep_cif_var(
    cif: *mut ffi_cif,
    abi: ffi_abi,
    nfixedargs: usize,
    ntotalargs: usize,
    rtype: *mut ffi_type,
    atypes: *mut *mut ffi_type,
) -> Result<()> {
    let status = raw::ffi_prep_cif_var(
        cif,
        abi,
        nfixedargs as c_uint,
        ntotalargs as c_uint,
        rtype,
        atypes,
    );
    status_to_result(status, ())
}

/// Calls a C function as specified by a CIF.
///
/// # Arguments
///
/// * `cif` — describes the argument and result types and the calling
///           convention
/// * `fun` — the function to call
/// * `args` — the arguments to pass to `fun`
///
/// # Result
///
/// The result of calling `fun` with `args`.
///
/// # Examples
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
///     prep_cif(&mut cif, ffi_abi_FFI_DEFAULT_ABI, 2,
///              &mut types::uint64, args.as_mut_ptr()).unwrap();
///
///     call::<u64>(&mut cif, CodePtr(c_function as *mut _),
///          vec![ &mut 4u64 as *mut _ as *mut c_void,
///                &mut 5u64 as *mut _ as *mut c_void ].as_mut_ptr())
/// };
///
/// assert_eq!(9, result);
/// ```
///
/// # Safety
/// libffi will read values from `args` based on the CIF, make sure that every
/// pointer points to correct data types that are properly aligned.
/// Additionally, the ffi function may perform actions that causes undefined
/// behavior. Extensive testing is recommended when dealing with ffi functions.
///
/// It is also important that the return type `R` matches the type of the value
/// returned from `fun` as a mismatch may lead to out-of-bounds reads, write,
/// and misaligned memory accesses.
pub unsafe fn call<R>(cif: *mut ffi_cif, fun: CodePtr, args: *mut *mut c_void) -> R {
    // libffi always writes *at least* a full register to the result pointer.
    // Therefore, if the return value is smaller, we need to handle the return
    // value with extra care to prevent out of bounds write and returning the
    // correct value in big endian architectures.
    //
    // There is no data type in rust that is guaranteed to be a full
    // register(?), but the assumption that usize is the full width of a
    // register holds for all tested architectures.
    if mem::size_of::<R>() < mem::size_of::<usize>() {
        // Alignments are a power of 2 (1, 2, 4, 8, etc). `result`'s alignment
        // is greater than or equal to that of `R`, so `result` should be
        // properly aligned for `R` since a larger alignment is always
        // divisible by any of the smaller alignments.
        let mut result = mem::MaybeUninit::<usize>::uninit();

        // SAFETY: It is up to the caller to ensure that the ffi_call is safe
        // to perform.
        unsafe {
            raw::ffi_call(
                cif,
                Some(*fun.as_safe_fun()),
                result.as_mut_ptr().cast::<c_void>(),
                args,
            );

            let result = result.assume_init();

            if cfg!(target_endian = "big") {
                call_return_small_big_endian_result((*(*cif).rtype).type_, &result)
            } else {
                (&result as *const usize).cast::<R>().read()
            }
        }
    } else {
        let mut result = mem::MaybeUninit::<R>::uninit();

        // SAFETY: It is up to the caller to ensure that the ffi_call is safe
        // to perform.
        unsafe {
            raw::ffi_call(
                cif,
                Some(*fun.as_safe_fun()),
                result.as_mut_ptr().cast::<c_void>(),
                args,
            );

            result.assume_init()
        }
    }
}

/// Helper function to get the return value of a ffi call on big endian
/// architectures.
///
/// # Safety
/// `result` must be a pointer to a `usize` and
/// `mem::size_of::<R> <= mem::size_of::<usize>()`.
unsafe fn call_return_small_big_endian_result<R>(type_tag: u16, result: *const usize) -> R {
    if type_tag == raw::FFI_TYPE_FLOAT as u16
        || type_tag == raw::FFI_TYPE_STRUCT as u16
        || type_tag == raw::FFI_TYPE_VOID as u16
    {
        // SAFETY: Testing has shown that these types appear at `result`.
        unsafe { result.cast::<R>().read() }
    } else {
        // SAFETY: Consider `*result` an array with
        // `size_of::<usize>() / size_of::<R>()` items of `R`. The following
        // code reads the last element to get the least significant bits of
        // `result` on big endian architectures. The most significant bits are
        // zeroed by libffi.
        unsafe {
            result
                .cast::<R>()
                .add((mem::size_of::<usize>() / mem::size_of::<R>()) - 1)
                .read()
        }
    }
}

/// Allocates a closure.
///
/// Returns a pair of the writable closure object and the function
/// pointer for calling it. The former acts as a handle to the closure,
/// and is used to configure and free it. The latter is the code pointer
/// used to invoke the closure. Before it can be invoked, it must be
/// initialized with [`prep_closure`] and [`prep_closure_mut`]. The
/// closure must be deallocated using [`closure_free`], after which
/// point the code pointer should not be used.
///
/// # Examples
///
/// ```
/// use libffi::low::*;
///
/// let (closure_handle, code_ptr) = closure_alloc();
/// ```
pub fn closure_alloc() -> (*mut ffi_closure, CodePtr) {
    unsafe {
        let mut code_pointer = mem::MaybeUninit::<*mut c_void>::uninit();
        let closure =
            raw::ffi_closure_alloc(mem::size_of::<ffi_closure>(), code_pointer.as_mut_ptr());
        (
            closure as *mut ffi_closure,
            CodePtr::from_ptr(code_pointer.assume_init()),
        )
    }
}

/// Frees a closure.
///
/// Closures allocated with [`closure_alloc`] must be deallocated with
/// [`closure_free`].
///
/// # Examples
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
pub type Callback<U, R> =
    unsafe extern "C" fn(cif: &ffi_cif, result: &mut R, args: *const *const c_void, userdata: &U);

/// The type of function called by a mutable closure.
///
/// `U` is the type of the user data captured by the closure and passed
/// to the callback, and `R` is the type of the result. The parameters
/// are not typed, since they are passed as a C array of `void*`.
pub type CallbackMut<U, R> = unsafe extern "C" fn(
    cif: &ffi_cif,
    result: &mut R,
    args: *const *const c_void,
    userdata: &mut U,
);

/// The callback type expected by [`raw::ffi_prep_closure_loc`].
pub type RawCallback = unsafe extern "C" fn(
    cif: *mut ffi_cif,
    result: *mut c_void,
    args: *mut *mut c_void,
    userdata: *mut c_void,
);

/// Initializes a closure with a callback function and userdata.
///
/// After allocating a closure with [`closure_alloc`], it needs to be
/// initialized with a function `callback` to call and a pointer
/// `userdata` to pass to it. Invoking the closure’s code pointer will
/// then pass the provided arguments and the user data pointer to the
/// callback.
///
/// For mutable userdata use [`prep_closure_mut`].
///
/// # Safety
///
/// The closure retains a reference to CIF `cif`, so that must
/// still be live when the closure is used lest undefined behavior
/// result.
///
/// # Arguments
///
/// - `closure` — the closure to initialize
/// - `cif` — the calling convention and types for calling the closure
/// - `callback` — the function that the closure will invoke
/// - `userdata` — the closed-over value, stored in the closure and
///    passed to the callback upon invocation
/// - `code` — the closure’s code pointer, *i.e.*, the second component
///   returned by [`closure_alloc`].
///
/// # Result
///
/// `Ok(())` for success or `Err(e)` for failure.
///
/// # Examples
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
///     prep_cif(&mut cif, ffi_abi_FFI_DEFAULT_ABI, 1, &mut types::uint64,
///              args.as_mut_ptr()).unwrap();
///
///     let (closure, code) = closure_alloc();
///     let add5: extern "C" fn(u64) -> u64 = mem::transmute(code);
///
///     prep_closure(closure,
///                  &mut cif,
///                  callback,
///                  &mut userdata,
///                  CodePtr(add5 as *mut _)).unwrap();
///
///     assert_eq!(11, add5(6));
///     assert_eq!(12, add5(7));
///
///     assert_eq!(22, twice(add5, 12));
/// }
/// ```
pub unsafe fn prep_closure<U, R>(
    closure: *mut ffi_closure,
    cif: *mut ffi_cif,
    callback: Callback<U, R>,
    userdata: *const U,
    code: CodePtr,
) -> Result<()> {
    let status = raw::ffi_prep_closure_loc(
        closure,
        cif,
        Some(mem::transmute::<Callback<U, R>, RawCallback>(callback)),
        userdata as *mut c_void,
        code.as_mut_ptr(),
    );
    status_to_result(status, ())
}

/// Initializes a mutable closure with a callback function and (mutable)
/// userdata.
///
/// After allocating a closure with [`closure_alloc`], it needs to be
/// initialized with a function `callback` to call and a pointer
/// `userdata` to pass to it. Invoking the closure’s code pointer will
/// then pass the provided arguments and the user data pointer to the
/// callback.
///
/// For immutable userdata use [`prep_closure`].
///
/// # Safety
///
/// The closure retains a reference to CIF `cif`, so that must
/// still be live when the closure is used lest undefined behavior
/// result.
///
/// # Arguments
///
/// - `closure` — the closure to initialize
/// - `cif` — the calling convention and types for calling the closure
/// - `callback` — the function that the closure will invoke
/// - `userdata` — the closed-over value, stored in the closure and
///    passed to the callback upon invocation
/// - `code` — the closure’s code pointer, *i.e.*, the second component
///   returned by [`closure_alloc`].
///
/// # Result
///
/// `Ok(())` for success or `Err(e)` for failure.
///
/// # Examples
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
///     prep_cif(&mut cif, ffi_abi_FFI_DEFAULT_ABI, 1, &mut types::uint64,
///              args.as_mut_ptr()).unwrap();
///
///     let (closure, code) = closure_alloc();
///     let add5: extern "C" fn(u64) -> u64 = mem::transmute(code);
///
///     prep_closure_mut(closure,
///                      &mut cif,
///                      callback,
///                      &mut userdata,
///                      CodePtr(add5 as *mut _)).unwrap();
///
///     assert_eq!(5, add5(6));
///     assert_eq!(11, add5(7));
///
///     assert_eq!(19, twice(add5, 1));
/// }
/// ```
pub unsafe fn prep_closure_mut<U, R>(
    closure: *mut ffi_closure,
    cif: *mut ffi_cif,
    callback: CallbackMut<U, R>,
    userdata: *mut U,
    code: CodePtr,
) -> Result<()> {
    let status = raw::ffi_prep_closure_loc(
        closure,
        cif,
        Some(mem::transmute::<CallbackMut<U, R>, RawCallback>(callback)),
        userdata as *mut c_void,
        code.as_mut_ptr(),
    );
    status_to_result(status, ())
}

#[cfg(test)]
mod test {
    use std::ptr::{addr_of_mut, null_mut};

    use super::*;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct SmallStruct(u8, u16);
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct LargeStruct(u64, u64, u64, u64);

    extern "C" fn return_nothing() {}
    extern "C" fn return_i8(a: i8) -> i8 {
        a
    }
    extern "C" fn return_u8(a: u8) -> u8 {
        a
    }
    extern "C" fn return_i16(a: i16) -> i16 {
        a
    }
    extern "C" fn return_u16(a: u16) -> u16 {
        a
    }
    extern "C" fn return_i32(a: i32) -> i32 {
        a
    }
    extern "C" fn return_u32(a: u32) -> u32 {
        a
    }
    extern "C" fn return_i64(a: i64) -> i64 {
        a
    }
    extern "C" fn return_u64(a: u64) -> u64 {
        a
    }
    extern "C" fn return_pointer(a: *const c_void) -> *const c_void {
        a
    }
    extern "C" fn return_f32(a: f32) -> f32 {
        a
    }
    extern "C" fn return_f64(a: f64) -> f64 {
        a
    }
    extern "C" fn return_small_struct(a: SmallStruct) -> SmallStruct {
        a
    }
    extern "C" fn return_large_struct(a: LargeStruct) -> LargeStruct {
        a
    }

    macro_rules! test_return_value {
        ($ty:ty, $ffitype:expr, $val:expr, $fn:ident) => {{
            let mut cif = ffi_cif::default();
            let mut arg_ty_array: [*mut ffi_type; 1] = [addr_of_mut!($ffitype)];
            let mut arg: $ty = $val;
            let mut arg_array: [*mut c_void; 1] = [addr_of_mut!(arg).cast()];

            prep_cif(
                &mut cif,
                ffi_abi_FFI_DEFAULT_ABI,
                1,
                addr_of_mut!($ffitype),
                arg_ty_array.as_mut_ptr(),
            )
            .unwrap();

            let result: $ty = call(&mut cif, CodePtr($fn as *mut _), arg_array.as_mut_ptr());

            assert_eq!(result, $val);
        }};
    }

    /// Test to ensure that values returned from functions called through libffi are correct.
    #[test]
    fn test_return_values() {
        // Test a function returning nothing.
        {
            let mut cif = ffi_cif::default();

            // SAFETY:
            // `cif` points to a properly aligned `ffi_cif`.
            // The return value is a pointer to an `ffi_type`.
            // `nargs` is 0, so argument and argument type array are never read.
            unsafe {
                prep_cif(
                    &mut cif,
                    ffi_abi_FFI_DEFAULT_ABI,
                    0,
                    addr_of_mut!(types::void),
                    null_mut(),
                )
                .unwrap();
                call::<()>(&mut cif, CodePtr(return_nothing as *mut _), null_mut());
            }
        }

        unsafe {
            test_return_value!(i8, types::sint8, 0x55, return_i8);
            test_return_value!(u8, types::uint8, 0xAA, return_u8);
            test_return_value!(i16, types::sint16, 0x5555, return_i16);
            test_return_value!(u16, types::uint16, 0xAAAA, return_u16);
            test_return_value!(i32, types::sint32, 0x5555_5555, return_i32);
            test_return_value!(u32, types::uint32, 0xAAAA_AAAA, return_u32);
            test_return_value!(i64, types::sint64, 0x5555_5555_5555_5555, return_i64);
            test_return_value!(u64, types::uint64, 0xAAAA_AAAA_AAAA_AAAA, return_u64);
            test_return_value!(f32, types::float, core::f32::consts::E, return_f32);
            test_return_value!(f64, types::double, core::f64::consts::PI, return_f64);

            let mut dummy = 0;
            test_return_value!(
                *const c_void,
                types::pointer,
                addr_of_mut!(dummy).cast(),
                return_pointer
            );

            let mut small_struct_elements = [
                addr_of_mut!(types::uint8),
                addr_of_mut!(types::uint16),
                null_mut(),
            ];
            let mut small_struct_type = ffi_type {
                type_: type_tag::STRUCT,
                elements: small_struct_elements.as_mut_ptr(),
                ..Default::default()
            };
            test_return_value!(
                SmallStruct,
                small_struct_type,
                SmallStruct(0xAA, 0x5555),
                return_small_struct
            );

            let mut large_struct_elements = [
                addr_of_mut!(types::uint64),
                addr_of_mut!(types::uint64),
                addr_of_mut!(types::uint64),
                addr_of_mut!(types::uint64),
                null_mut(),
            ];
            let mut large_struct_type = ffi_type {
                type_: type_tag::STRUCT,
                elements: large_struct_elements.as_mut_ptr(),
                ..Default::default()
            };
            test_return_value!(
                LargeStruct,
                large_struct_type,
                LargeStruct(
                    0x1234_5678_9abc_def0,
                    0x0fed_cba9_8765_4321,
                    0x5555_5555_5555_5555,
                    0xAAAA_AAAA_AAAA_AAAA,
                ),
                return_large_struct
            );
        }
    }
}
