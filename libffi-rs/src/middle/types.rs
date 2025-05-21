//! Representations of C types and arrays thereof.
//!
//! These are used to describe the types of the arguments and results of
//! functions. When we construct a [CIF](super::Cif) (“Call
//! Inter<span></span>Face”), we provide a sequence of argument types
//! and a result type, and libffi uses this to figure out how to set up
//! a call to a function with those types.

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;

use core::iter::once;
use core::ptr::{addr_of_mut, null_mut};
use core::{fmt, slice};

use crate::low::{ffi_type, type_tag};

// Use types defined in Rust when executing miri
#[cfg(all(miri, feature = "complex", not(windows)))]
use miri::{complex_double, complex_float, complex_longdouble};
#[cfg(miri)]
use miri::{
    double, float, longdouble, pointer, sint16, sint32, sint64, sint8, uint16, uint32, uint64,
    uint8, void,
};

#[cfg(all(not(miri), feature = "complex", not(windows)))]
use crate::low::types::{complex_double, complex_float, complex_longdouble};
#[cfg(not(miri))]
use crate::low::types::{
    double, float, longdouble, pointer, sint16, sint32, sint64, sint8, uint16, uint32, uint64,
    uint8, void,
};

/// Represents a single C type.
///
/// # Example
///
/// Suppose we have a C struct:
///
/// ```c
/// struct my_struct {
///     uint16_t f1;
///     uint64_t f2;
/// };
/// ```
///
/// To pass the struct by value via libffi, we need to construct a
/// `Type` object describing its layout:
///
/// ```
/// use libffi::middle::Type;
///
/// let my_struct = Type::structure([
///     Type::U64,
///     Type::U16,
/// ]);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    /// Represents a `i8`
    I8,
    /// Represents a `u8`
    U8,
    /// Represents a `i16`
    I16,
    /// Represents a `u16`
    U16,
    /// Represents a `i32`
    I32,
    /// Represents a `u32`
    U32,
    /// Represents a `i64`
    I64,
    /// Represents a `u64`
    U64,
    /// Represents a `isize`
    Isize,
    /// Represents a `usize`
    Usize,
    /// Represents a pointer
    Pointer,
    /// Represents a `f32`
    F32,
    /// Represents a `f64`
    F64,
    /// Represents a `long double`.
    LongDouble,
    /// Returns the C `_Complex float` type.
    ///
    /// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
    #[cfg(all(feature = "complex", not(windows)))]
    ComplexFloat,
    /// Returns the C `_Complex double` type.
    ///
    /// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
    #[cfg(all(feature = "complex", not(windows)))]
    ComplexDouble,
    /// Returns the C `_Complex double` type.
    ///
    /// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
    #[cfg(all(feature = "complex", not(windows)))]
    ComplexLongDouble,
    /// Represents a `repr(C)` structure.
    ///
    /// It is recommended to create a `Type::Structure` using [`Type::structure`].
    Structure(Box<[Type]>),
}

const fn signed_data_to_scalar_type<T: Sized>() -> Type {
    match core::mem::size_of::<T>() {
        1 => Type::I8,
        2 => Type::I16,
        4 => Type::I32,
        8 => Type::I64,
        _ => panic!("Unsupported int size"),
    }
}

const fn unsigned_data_to_scalar_type<T: Sized>() -> Type {
    match core::mem::size_of::<T>() {
        1 => Type::U8,
        2 => Type::U16,
        4 => Type::U32,
        8 => Type::U64,
        _ => panic!("Unsupported int size"),
    }
}

impl Type {
    /// Returns the `Type` that corresponds to `c_schar`.
    pub const C_SCHAR: Self = signed_data_to_scalar_type::<core::ffi::c_schar>();
    /// Returns the `Type` that corresponds to `c_uchar`.
    pub const C_UCHAR: Self = signed_data_to_scalar_type::<core::ffi::c_uchar>();
    /// Returns the `Type` that corresponds to `c_short`.
    pub const C_SHORT: Self = signed_data_to_scalar_type::<core::ffi::c_short>();
    /// Returns the `Type` that corresponds to `c_ushort`.
    pub const C_USHORT: Self = signed_data_to_scalar_type::<core::ffi::c_ushort>();
    /// Returns the `Type` that corresponds to `c_int`.
    pub const C_INT: Self = signed_data_to_scalar_type::<core::ffi::c_int>();
    /// Returns the `Type` that corresponds to `c_uint`.
    pub const C_UINT: Self = signed_data_to_scalar_type::<core::ffi::c_uint>();
    /// Returns the `Type` that corresponds to `c_long`.
    pub const C_LONG: Self = signed_data_to_scalar_type::<core::ffi::c_long>();
    /// Returns the `Type` that corresponds to `c_ulong`.
    pub const C_ULONG: Self = signed_data_to_scalar_type::<core::ffi::c_ulong>();
    /// Returns the `Type` that corresponds to `c_longlong`.
    pub const C_LONGLONG: Self = signed_data_to_scalar_type::<core::ffi::c_longlong>();
    /// Returns the `Type` that corresponds to `c_ulonglong`.
    pub const C_ULONGLONG: Self = signed_data_to_scalar_type::<core::ffi::c_ulonglong>();

    /// Returns the signed 8-bit numeric type.
    #[deprecated = "Refer to `Type::I8` directly. This function will be removed in a future version."]
    pub const fn i8() -> Self {
        Self::I8
    }

    /// Returns the unsigned 8-bit numeric type.
    #[deprecated = "Refer to `Type::U8` directly. This function will be removed in a future version."]
    pub const fn u8() -> Self {
        Self::U8
    }

    /// Returns the signed 16-bit numeric type.
    #[deprecated = "Refer to `Type::I16` directly. This function will be removed in a future version."]
    pub const fn i16() -> Self {
        Self::I16
    }

    /// Returns the unsigned 16-bit numeric type.
    #[deprecated = "Refer to `Type::U16` directly. This function will be removed in a future version."]
    pub const fn u16() -> Self {
        Self::U16
    }

    /// Returns the signed 32-bit numeric type.
    #[deprecated = "Refer to `Type::I32` directly. This function will be removed in a future version."]
    pub const fn i32() -> Self {
        Self::I32
    }

    /// Returns the unsigned 32-bit numeric type.
    #[deprecated = "Refer to `Type::U32` directly. This function will be removed in a future version."]
    pub const fn u32() -> Self {
        Self::U32
    }
    /// Returns the signed 64-bit numeric type.
    #[deprecated = "Refer to `Type::I64` directly. This function will be removed in a future version."]
    pub const fn i64() -> Self {
        Self::I64
    }

    /// Returns the unsigned 64-bit numeric type.
    #[deprecated = "Refer to `Type::U64` directly. This function will be removed in a future version."]
    pub const fn u64() -> Self {
        Self::U64
    }

    /// Returns the C equivalent of Rust `isize` (`i64`).
    #[deprecated = "Refer to `Type::Isize` directly. This function will be removed in a future version."]
    pub const fn isize() -> Self {
        Self::Isize
    }

    /// Returns the C equivalent of Rust `usize` (`u64`).
    #[deprecated = "Refer to `Type::Usize` directly. This function will be removed in a future version."]
    pub const fn usize() -> Self {
        Self::Usize
    }

    /// Returns the C `void*` type, for passing any kind of pointer.
    #[deprecated = "Refer to `Type::Pointer` directly. This function will be removed in a future version."]
    pub const fn pointer() -> Self {
        Self::Pointer
    }

    /// Returns the C `signed char` type.
    #[deprecated = "Use `Type::C_SCHAR` instead. This function will be removed in a future version."]
    pub const fn c_schar() -> Self {
        signed_data_to_scalar_type::<core::ffi::c_schar>()
    }

    /// Returns the C `unsigned char` type.
    #[deprecated = "Use `Type::C_UCHAR` instead. This function will be removed in a future version."]
    pub const fn c_uchar() -> Self {
        unsigned_data_to_scalar_type::<core::ffi::c_uchar>()
    }

    /// Returns the C `short` type.
    #[deprecated = "Use `Type::C_SHORT` instead. This function will be removed in a future version."]
    pub const fn c_short() -> Self {
        signed_data_to_scalar_type::<core::ffi::c_short>()
    }

    /// Returns the C `unsigned short` type.
    #[deprecated = "Use `Type::C_USHORT` instead. This function will be removed in a future version."]
    pub const fn c_ushort() -> Self {
        unsigned_data_to_scalar_type::<core::ffi::c_ushort>()
    }

    /// Returns the C `int` type.
    #[deprecated = "Use `Type::C_INT` instead. This function will be removed in a future version."]
    pub const fn c_int() -> Self {
        signed_data_to_scalar_type::<core::ffi::c_int>()
    }

    /// Returns the C `unsigned int` type.
    #[deprecated = "Use `Type::C_UINT` instead. This function will be removed in a future version."]
    pub const fn c_uint() -> Self {
        unsigned_data_to_scalar_type::<core::ffi::c_uint>()
    }

    /// Returns the C `long` type.
    #[deprecated = "Use `Type::LONG` instead. This function will be removed in a future version."]
    pub const fn c_long() -> Self {
        signed_data_to_scalar_type::<core::ffi::c_long>()
    }

    /// Returns the C `unsigned long` type.
    #[deprecated = "Use `Type::ULONG` instead. This function will be removed in a future version."]
    pub const fn c_ulong() -> Self {
        unsigned_data_to_scalar_type::<core::ffi::c_ulong>()
    }

    /// Returns the C `longlong` type.
    #[deprecated = "Use `Type::LONGLONG` instead. This function will be removed in a future version."]
    pub const fn c_longlong() -> Self {
        signed_data_to_scalar_type::<core::ffi::c_longlong>()
    }

    /// Returns the C `unsigned longlong` type.
    #[deprecated = "Use `Type::ULONGLONG` instead. This function will be removed in a future version."]
    pub const fn c_ulonglong() -> Self {
        unsigned_data_to_scalar_type::<core::ffi::c_ulonglong>()
    }

    /// Returns the C `float` (32-bit floating point) type.
    #[deprecated = "Refer to `Type::F32` directly. This function will be removed in a future version."]
    pub const fn f32() -> Self {
        Self::F32
    }

    /// Returns the C `double` (64-bit floating point) type.
    #[deprecated = "Refer to `Type::F64` directly. This function will be removed in a future version."]
    pub const fn f64() -> Self {
        Self::F64
    }

    /// Returns the C `long double` (extended-precision floating point) type.
    #[deprecated = "Refer to `Type::LongDouble` directly. This function will be removed in a future version."]
    pub const fn longdouble() -> Self {
        Self::LongDouble
    }

    /// Returns the C `_Complex float` type.
    ///
    /// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
    #[cfg(all(feature = "complex", not(windows)))]
    #[deprecated = "Refer to `Type::ComplexFloat` directly. This function will be removed in a future version."]
    pub const fn c32() -> Self {
        Self::ComplexFloat
    }

    /// Returns the C `_Complex double` type.
    ///
    /// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
    #[cfg(all(feature = "complex", not(windows)))]
    #[deprecated = "Refer to `Type::ComplexDouble` directly. This function will be removed in a future version."]
    pub const fn c64() -> Self {
        Self::ComplexDouble
    }

    /// Returns the C `_Complex long double` type.
    ///
    /// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
    #[cfg(all(feature = "complex", not(windows)))]
    #[deprecated = "Refer to `Type::ComplexLongDouble` directly. This function will be removed in a future version."]
    pub const fn complex_longdouble() -> Self {
        Self::ComplexLongDouble
    }

    /// Constructs a structure type whose fields have the given types.
    ///
    /// # Example
    ///
    /// Creating a `Type` for the following C structure:
    ///
    /// ```c
    /// struct ForeignStruct {
    ///     uint64_t id;
    ///     void *ptr;
    /// };
    /// ```
    ///
    /// ```
    /// use libffi::middle::Type;
    ///
    /// let ty = Type::structure([
    ///     Type::U64,
    ///     Type::Pointer,
    /// ]);
    /// ```
    pub fn structure<I>(fields: I) -> Self
    where
        I: IntoIterator<Item = Type>,
    {
        Self::Structure(fields.into_iter().collect())
    }
}

/// Type used to manage an array of `FfiType`s for a [`Cif`].
///
/// This is only intended to be used to manage the array for a [`Cif`]'s
/// argument types. The pointer this struct contains is only intended to pass on
/// to libffi, and this crate makes no guarantees that it is safe to create
/// references from the pointer.
///
/// # Usage restrictions
///
/// * `FfiTypeArray` should only be owned and used by **one** [`Cif`].
/// * [`FfiTypeArray::as_ffi_ptr`] should **only** be used to store the array in
///   a [`Cif`] so it can be used by libffi.
/// * There are further restrictions on [`FfiType`].
///
/// [`Cif`]: `crate::middle::Cif`
#[repr(transparent)]
pub(crate) struct FfiTypeArray(*mut [FfiType]);

impl FfiTypeArray {
    /// Create a new `FfiTypeArray` from `Type`s.
    pub fn new<I>(types: I) -> Self
    where
        I: IntoIterator<Item = Type>,
    {
        let type_slice: Box<[FfiType]> = types.into_iter().map(|ty| FfiType::new(&ty)).collect();

        let type_slice_ptr = Box::into_raw(type_slice);

        Self(type_slice_ptr)
    }

    /// Get the length of the slice.
    ///
    /// # Safety
    ///
    /// This function creates a `&[FfiType]` reference from the `*mut [FfiType]`
    /// pointer in `self.0`. The following rules must be followed:
    ///
    /// * `self.0` must be a valid, properly aligned, non-null pointer
    /// * `self.0` must not be mutated while `len` is being called
    pub unsafe fn len(&self) -> usize {
        // When MSRV is increased to >= 1.79, this can be changed to
        // `self.0.len()`. That change should make this function safe as there
        // will no longer be a need to create a reference from a pointer.
        #[allow(clippy::needless_borrow)]
        unsafe {
            (&(*self.0)).len()
        }
    }

    /// Get a pointer to the underlying `ffi_type` array.
    ///
    /// Note that the pointer can only be used as long as `self` is alive. The
    /// pointer should not be used for anything other than storing it in a
    /// [`Cif`] which is then passed to libffi.
    ///
    /// [`Cif`]: `crate::middle::Cif`
    pub fn as_ffi_ptr(&self) -> *mut *mut ffi_type {
        unsafe { (*self.0).as_mut_ptr().cast::<*mut ffi_type>() }
    }
}

impl Clone for FfiTypeArray {
    fn clone(&self) -> Self {
        // Create a new boxed slice with a clone of every `FfiType` stored in
        // this `FfiTypeArray`. Note that `self.0` is a fat pointer that stores
        // the number of elements in the slice, which is why we can simply
        // iterate over `*self.0`.
        let slice_clone: Box<[FfiType]> = unsafe { (*self.0).iter() }.cloned().collect();

        let slice_clone_ptr = Box::into_raw(slice_clone);

        Self(slice_clone_ptr)
    }
}

impl Drop for FfiTypeArray {
    fn drop(&mut self) {
        // Convert `self.0` back to the boxed slice to deallocate the memory
        // allocated for the slice.
        let _drop = unsafe { Box::from_raw(self.0) };
    }
}

impl core::fmt::Debug for FfiTypeArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(unsafe { &(*self.0) }).finish()
    }
}

/// Type used to manage `ffi_type`s used in the `middle` module.
///
/// `FfiType` owns and is responsible for managing the memory of its `ffi_type`.
/// For structs `FfiType` also allocates the memory required for the
/// `ffi_type`'s `elements` field, and is responsible for freeing it when the
/// `FfiType` is dropped. On PowerPC, `long double`s and complex numbers are
/// handled in the same way as structs to avoid thread safety issues.
///
/// # Usage restrictions
///
/// The following restrictions must be followed to ensure safe and sound usage
/// of `FfiType`:
///
/// * `FfiType` should only be owned and used by **one** [`Cif`],
///   [`FfiTypeArray`], or [`FfiType`].
/// * `ffi_prep_cif` must be called from the [`Cif`] using this `FfiType` before
///   it can be used to make function calls. It is not necessarily safe to
///   substitute an `FfiType` with another even though they represent identical
///   types.
/// * [`FfiType::as_ffi_ptr`] should **only** be used to pass a pointer to C
///   libffi functions. This crate does not guarantee that it is sound to create
///   references from `FfiType`'s pointers.
///   * Libffi may write to the `ffi_type` on initialization, but according to
///     its documentation, will not perform any modifications after
///     initialization.
///
/// Thread safety restrictions are based on [libffi's manual][threadsafety-url].
///
/// # Complex number structure
///
/// Complex numbers' `elements` array consists of a pointer to a `ffi_type` that
/// defines the type of numbers stored in the complex type followed by a NULL.
/// In Rust, the `elements` array for `complex_float` would look like this:
///
/// ```
/// use libffi::low::types::float;
/// use std::ptr::{addr_of_mut, null_mut};
///
/// let complex_float_elements = [unsafe { addr_of_mut!(float) }, null_mut()];
/// ```
///
/// For the definition of the built-in complex types, see
/// https://github.com/libffi/libffi/blob/v3.4.7/src/types.c#L102.
///
/// [threadsafety-url]: https://github.com/libffi/libffi/blob/v3.4.7/doc/libffi.texi#L949
#[repr(transparent)]
pub(crate) struct FfiType(*mut ffi_type);

// Required as Rust 1.78 needs unsafe blocks when getting pointers to
// `static mut`s. When MSRV is bumped to 1.82 or higher, this can be dropped in
// favor of `&raw mut` to get the pointers.
#[allow(unused_unsafe)]
impl FfiType {
    const IS_POWERPC: bool = cfg!(any(target_arch = "powerpc", target_arch = "powerpc64"));

    /// Get a `FfiType` for a `void` return type.
    ///
    /// This needs to be a function and not a constant as `const_refs_to_static`
    /// is not stable on Rust 1.78.
    pub fn void() -> Self {
        Self(unsafe { addr_of_mut!(void) })
    }

    /// Creates a new `FfiType` from a [`Type`].
    ///
    /// `ffi_prep_cif` must be called from the [`Cif`] using this `FfiType`
    /// before it can be used to make function calls. It is not necessarily safe
    /// to substitute an `FfiType` with another even though they represent
    /// identical types.
    ///
    /// For basic data types provided by libffi (integers, floats, pointers), no
    /// allocation is needed. Instead, the static `ffi_type`s exported by libffi
    /// are used.
    ///
    /// On PowerPC, `long double`s and complex numbers are cloned to ensure that
    /// there are no thread safety issues.
    ///
    /// For structs, a new `ffi_type` is allocated along with an array of
    /// `ffi_type` elements.
    pub fn new(ty: &Type) -> Self {
        match (ty, Self::IS_POWERPC) {
            (Type::I8, _) => Self(unsafe { addr_of_mut!(sint8) }),
            (Type::U8, _) => Self(unsafe { addr_of_mut!(uint8) }),
            (Type::I16, _) => Self(unsafe { addr_of_mut!(sint16) }),
            (Type::U16, _) => Self(unsafe { addr_of_mut!(uint16) }),
            (Type::I32, _) => Self(unsafe { addr_of_mut!(sint32) }),
            (Type::U32, _) => Self(unsafe { addr_of_mut!(uint32) }),
            (Type::I64, _) => Self(unsafe { addr_of_mut!(sint64) }),
            (Type::U64, _) => Self(unsafe { addr_of_mut!(uint64) }),
            (Type::Isize, _) => {
                #[cfg(target_pointer_width = "16")]
                {
                    Self(unsafe { addr_of_mut!(sint16) })
                }
                #[cfg(target_pointer_width = "32")]
                {
                    Self(unsafe { addr_of_mut!(sint32) })
                }
                #[cfg(target_pointer_width = "64")]
                {
                    Self(unsafe { addr_of_mut!(sint64) })
                }
            }
            (Type::Usize, _) => {
                #[cfg(target_pointer_width = "16")]
                {
                    Self(unsafe { addr_of_mut!(uint16) })
                }
                #[cfg(target_pointer_width = "32")]
                {
                    Self(unsafe { addr_of_mut!(uint32) })
                }
                #[cfg(target_pointer_width = "64")]
                {
                    Self(unsafe { addr_of_mut!(uint64) })
                }
            }
            (Type::Pointer, _) => Self(unsafe { addr_of_mut!(pointer) }),
            (Type::F32, _) => Self(unsafe { addr_of_mut!(float) }),
            (Type::F64, _) => Self(unsafe { addr_of_mut!(double) }),

            // When not on PowerPC we can simply use libffi's `longdouble`.
            (Type::LongDouble, false) => Self(unsafe { addr_of_mut!(longdouble) }),
            // On PowerPC, `longdouble` is not thread safe and must be handled
            // accordingly, with a clone for each `FfiType`.
            (Type::LongDouble, true) => {
                // SAFETY:
                // `longdouble` is a type defined by libffi. It may hold
                // incorrect values for size and alignment, but this should be
                // fixed by `ffi_prep_cif`.
                unsafe { Self::clone_from_ffi_type(unsafe { addr_of_mut!(longdouble) }) }
            }

            // When not on PowerPC we can simply use libffi's `complex_float`.
            #[cfg(all(feature = "complex", not(windows)))]
            (Type::ComplexFloat, false) => Self(unsafe { addr_of_mut!(complex_float) }),
            // On PowerPC, `longdouble` is not thread safe. Therefore we need to
            // handle complex types specially because they need an `elements`
            // array that **might** contain a `longdouble`.
            //
            // While it is not strictly required to clone the first `elements`
            // item for a complex float, it is done to keep things simpler and
            // to avoid potential bugs when dropping the `FfiType`.
            #[cfg(all(feature = "complex", not(windows)))]
            (Type::ComplexFloat, true) => {
                // SAFETY:
                // `complex_float` is a type defined by libffi.
                unsafe { Self::clone_from_ffi_type(addr_of_mut!(complex_float)) }
            }

            // When not on PowerPC we can simply use libffi's `complex_double`.
            #[cfg(all(feature = "complex", not(windows)))]
            (Type::ComplexDouble, false) => Self(unsafe { addr_of_mut!(complex_double) }),
            // On PowerPC, `longdouble` is not thread safe. Therefore we need to
            // handle complex types specially because they need an `elements`
            // array that **might** contain a `longdouble`.
            //
            // While it is not strictly required to clone the first `elements`
            // item for a complex double, it is done to keep things simpler and
            // to avoid potential bugs when dropping the `FfiType`.
            #[cfg(all(feature = "complex", not(windows)))]
            (Type::ComplexDouble, true) => {
                // SAFETY:
                // `complex_double` is a type defined by libffi.
                unsafe { Self::clone_from_ffi_type(addr_of_mut!(complex_double)) }
            }

            // When not on PowerPC we can simply use libffi's `complex_longdouble`.
            #[cfg(all(feature = "complex", not(windows)))]
            (Type::ComplexLongDouble, false) => Self(unsafe { addr_of_mut!(complex_longdouble) }),
            // On PowerPC, `longdouble` is not thread safe. Therefore we need to
            // handle complex types specially because they need an `elements`
            // array that **might** contain a `longdouble`.
            #[cfg(all(feature = "complex", not(windows)))]
            (Type::ComplexLongDouble, true) => {
                // SAFETY:
                // `complex_longdouble` is a type defined by libffi. It may hold
                // incorrect values for size and alignment, but this should be
                // fixed by `ffi_prep_cif`.
                unsafe { Self::clone_from_ffi_type(addr_of_mut!(longdouble)) }
            }

            (Type::Structure(items), _) => {
                // Create `FfiType`s for every element in this struct.
                let elements = items
                    .iter()
                    // Convert every `Type` to `FfiType`
                    .map(FfiType::new)
                    // Append a NULL pointer to the end of list so that libffi
                    // knows there are no more elements.
                    .chain(once(FfiType(null_mut())))
                    .collect::<Box<[FfiType]>>();

                let mut inner_type = Box::<ffi_type>::default();

                // If we have not panicked so far, we *should* not panic until
                // this function returns, in which case `FfiType`'s `Drop`
                // implementation should take care of freeing the memory we have
                // allocated.

                inner_type.type_ = type_tag::STRUCT;
                // Casting from `*mut [FfiType]` to `*mut *mut ffi_type` is okay
                // here as `FfiType` is `#[repr(transparent)]` and only contains
                // a single `*mut ffi_type`. This is therefore a conversion from
                // `*mut [*mut ffi_type]` to `*mut *mut ffi_type`, where the
                // outermost pointer points to the first `*mut ffi_type` in the
                // slice.
                //
                // The last pointer (NULL) should prevent libffi from reading
                // out of bounds.
                inner_type.elements = Box::into_raw(elements).cast::<*mut ffi_type>();
                // Size and alignment will be set by `ffi_prep_cif`.

                Self(Box::into_raw(inner_type))
            }
        }
    }

    /// Get a pointer to the underlying `ffi_type`.
    ///
    /// Note that the pointer can only be used as long as `self` is alive. The
    /// pointer should not be used for anything other than storing it in a
    /// `Cif`, `FfiTypeArray`, or another `FfiType`.
    pub fn as_ffi_ptr(&self) -> *mut ffi_type {
        self.0
    }

    /// Helper function to create a new `FfiType` from a `*mut ffi_type`.
    ///
    /// # Safety
    ///
    /// The `ffi_type` must be well-formed. This function should only be called
    /// on `ffi_type`s created by libffi or `FfiType`.
    unsafe fn clone_from_ffi_type(ty: *mut ffi_type) -> Self {
        let mut ty_clone = Box::new(*ty);

        // If `elements` is `NULL` there is nothing more that needs to be done.
        if ty_clone.elements.is_null() {
            return Self(Box::into_raw(ty_clone));
        }

        // If `elements` is not `NULL` we need to clone each member of
        // `elements` and allocate space to store the array.

        let mut elements_vec: Vec<FfiType> = Vec::new();

        unsafe {
            let mut element_ptr = ty_clone.elements;
            while !(*element_ptr).is_null() {
                // Casting `*mut *mut ffi_type` to `*mut FfiType`. `FfiType` is
                // `#[repr(transparent)]` with a single `*mut ffi_type` item.
                let cloned_element = (*(element_ptr.cast::<FfiType>())).clone();

                elements_vec.push(cloned_element);
                element_ptr = element_ptr.add(1);
            }
        }

        elements_vec.push(Self(null_mut()));

        let elements = elements_vec.into_boxed_slice();

        // If we have not panicked so far, we *should* not panic until this
        // function returns, in which case `FfiType`'s `Drop` implementation
        // should take care of freeing the memory we have allocated.

        let slice_ptr = Box::into_raw(elements);
        let elements_ptr = unsafe { (*slice_ptr).as_mut_ptr() };
        // Casting from `*mut FfiType` to `*mut *mut ffi_type` is okay as
        // `FfiType` is `#[repr(transparent)]` and only contains a single
        // `*mut ffi_type`.
        ty_clone.elements = elements_ptr.cast::<*mut ffi_type>();

        Self(Box::into_raw(ty_clone))
    }

    /// Helper function to drop `Box`es owned by `self`.
    ///
    /// # Safety
    ///
    /// This function should only be called on structs on most architectures. On
    /// PowerPC it should also be called on `long double`s and complex numbers.
    ///
    /// Do not call this function manually, it should only be called from
    /// `FfiType`'s `Drop` implementation.
    unsafe fn deallocate_boxes(&mut self) {
        let self_ptr = self.0;
        let self_box = unsafe { Box::from_raw(self_ptr) };

        if !self_box.elements.is_null() {
            unsafe {
                // Initialize n_args to 1 as the while loop below does not count
                // the last `NULL`.
                let mut n_args = 1;

                let mut element = self_box.elements;
                while !(*element).is_null() {
                    element = element.add(1);
                    n_args += 1;
                }

                // Reconstruct the `Box` with the `elements` array. When it is
                // dropped, the `FfiType`s in the array will also be dropped.
                let _elements_box = Box::<[FfiType]>::from_raw(slice::from_raw_parts_mut(
                    self_box.elements.cast::<FfiType>(),
                    n_args,
                ));
            }
        }
    }
}

impl Clone for FfiType {
    fn clone(&self) -> Self {
        if self.0.is_null() {
            return Self(null_mut());
        }

        // If `self.0` is not NULL, it should point to a valid `ffi_type`.
        let self_type_tag = unsafe { (*self.0).type_ };

        match (self_type_tag, Self::IS_POWERPC) {
            (type_tag::STRUCT, _) | (type_tag::LONGDOUBLE, true) => {
                // SAFETY:
                // The `ffi_type` was created by `FfiType`.
                unsafe { Self::clone_from_ffi_type(self.0) }
            }

            #[cfg(all(feature = "complex", not(windows)))]
            (type_tag::COMPLEX, true) => {
                // SAFETY:
                // The `ffi_type` was created by `FfiType`.
                unsafe { Self::clone_from_ffi_type(self.0) }
            }

            _ => {
                // If nothing has matched no cloning is necessary
                Self(self.0)
            }
        }
    }
}

impl Drop for FfiType {
    fn drop(&mut self) {
        if self.0.is_null() {
            // If `self.0` is `NULL`, there is nothing to drop.
            return;
        }

        // If `self.0` is not `NULL`, it should point to a valid `ffi_type`.
        let self_type_tag = unsafe { (*self.0).type_ };

        match (self_type_tag, Self::IS_POWERPC) {
            (type_tag::STRUCT, _) | (type_tag::LONGDOUBLE, true) => {
                // SAFETY:
                // Either a struct or a `long double` on PowerPC.
                unsafe { self.deallocate_boxes() }
            }

            #[cfg(all(feature = "complex", not(windows)))]
            (type_tag::COMPLEX, true) => {
                // SAFETY:
                // A complex on PowerPC.
                unsafe { self.deallocate_boxes() }
            }

            _ => {
                // If this is a type created by libffi, we should not do anything.
            }
        }
    }
}

impl core::fmt::Debug for FfiType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_null() {
            f.write_str("FfiType(NULL)")
        } else {
            // If `FfiType` is not NULL it should point to a valid `ffi_type`
            unsafe { (*self.0).fmt(f) }
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod test {
    use alloc::format;

    use super::*;
    use crate::raw;

    #[test]
    fn create_u64() {
        FfiType::new(&Type::U64);
    }

    #[test]
    fn clone_u64() {
        let _ = Type::U64.clone().clone();
    }

    #[test]
    fn create_struct() {
        Type::structure(alloc::vec![Type::I64, Type::I64, Type::U64]);
    }

    #[test]
    fn clone_struct() {
        let _ = Type::structure(alloc::vec![Type::I64, Type::I64, Type::U64])
            .clone()
            .clone();
    }

    /// Verify that [`Type`]'s `Debug` impl does not misbehave.
    #[test]
    fn verify_type_debug_behavior() {
        let ffi_type = Type::structure([
            Type::U16,
            Type::F32,
            Type::structure([Type::I32, Type::structure([])]),
            Type::Pointer,
        ]);

        let _string = format!("{ffi_type:?}");
    }

    #[test]
    fn verify_type_layout() {
        let type_struct = Type::structure([
            // First struct, containing a struct, i8, and u8
            Type::structure([
                // Second struct, containing a i16, struct, and u16
                Type::I16,
                Type::structure([
                    // Third struct, containing a i32, u32 and struct
                    Type::I32,
                    Type::U32,
                    Type::structure([
                        // Fourth struct, only a struct
                        Type::structure([
                            // Fifth and final struct, no members
                        ]),
                    ]),
                ]),
                Type::U16,
            ]),
            Type::I8,
            Type::U8,
            Type::F32,
            Type::F64,
            #[cfg(all(feature = "complex", not(windows)))]
            Type::ComplexFloat,
            #[cfg(all(feature = "complex", not(windows)))]
            Type::ComplexDouble,
        ]);

        let ffi_struct = FfiType::new(&type_struct);

        verify_struct_layout(&ffi_struct);

        let clone1 = ffi_struct.clone();
        verify_struct_layout(&ffi_struct);
        verify_struct_layout(&clone1);

        let clone2 = ffi_struct.clone();
        verify_struct_layout(&ffi_struct);
        verify_struct_layout(&clone1);
        verify_struct_layout(&clone2);

        let clone3 = clone1.clone();
        verify_struct_layout(&ffi_struct);
        verify_struct_layout(&clone1);
        verify_struct_layout(&clone2);
        verify_struct_layout(&clone3);

        drop(clone2);
        verify_struct_layout(&ffi_struct);
        verify_struct_layout(&clone1);
        verify_struct_layout(&clone3);

        drop(ffi_struct);
        verify_struct_layout(&clone1);
        verify_struct_layout(&clone3);
    }

    // Utility function to verify the layout of the struct created by
    // `verify_raw_type_layout` and to ensure proper memory handling when
    // cloning and dropping the `Type`.
    fn verify_struct_layout(ty: &FfiType) {
        // First struct: struct, i8, u8
        let struct_1 = unsafe { &*ty.0 };
        assert_eq!(struct_1.size, 0);
        assert_eq!(struct_1.alignment, 0);
        assert_eq!(struct_1.type_, raw::FFI_TYPE_STRUCT);

        assert_eq!(unsafe { (**struct_1.elements).type_ }, raw::FFI_TYPE_STRUCT);
        assert_eq!(
            unsafe { (**struct_1.elements.add(1)).type_ },
            raw::FFI_TYPE_SINT8
        );
        assert_eq!(
            unsafe { (**struct_1.elements.add(2)).type_ },
            raw::FFI_TYPE_UINT8
        );
        assert_eq!(
            unsafe { (**struct_1.elements.add(3)).type_ },
            raw::FFI_TYPE_FLOAT
        );
        assert_eq!(
            unsafe { (**struct_1.elements.add(4)).type_ },
            raw::FFI_TYPE_DOUBLE
        );
        #[cfg(any(not(feature = "complex"), windows))]
        assert!(unsafe { (*struct_1.elements.add(5)).is_null() });
        #[cfg(all(feature = "complex", not(windows)))]
        assert_eq!(
            unsafe { (**struct_1.elements.add(5)).type_ },
            raw::FFI_TYPE_COMPLEX
        );
        #[cfg(all(feature = "complex", not(windows)))]
        assert_eq!(
            unsafe { (**struct_1.elements.add(6)).type_ },
            raw::FFI_TYPE_COMPLEX
        );
        #[cfg(all(feature = "complex", not(windows)))]
        assert_eq!(
            unsafe { (**struct_1.elements.add(7)).type_ },
            raw::FFI_TYPE_COMPLEX
        );
        #[cfg(all(feature = "complex", not(windows)))]
        assert!(unsafe { (*struct_1.elements.add(8)).is_null() });

        // Second struct: i16, struct, u16
        let struct_2 = unsafe { &**struct_1.elements };
        assert_eq!(struct_2.size, 0);
        assert_eq!(struct_2.alignment, 0);
        assert_eq!(struct_2.type_, raw::FFI_TYPE_STRUCT);

        assert_eq!(unsafe { (**struct_2.elements).type_ }, raw::FFI_TYPE_SINT16);
        assert_eq!(
            unsafe { (**struct_2.elements.add(1)).type_ },
            raw::FFI_TYPE_STRUCT
        );
        assert_eq!(
            unsafe { (**struct_2.elements.add(2)).type_ },
            raw::FFI_TYPE_UINT16
        );
        assert!(unsafe { (*struct_2.elements.add(3)).is_null() });

        // Third struct: i8, u8, struct
        let struct_3 = unsafe { &**(struct_2.elements.add(1)) };
        assert_eq!(struct_3.size, 0);
        assert_eq!(struct_3.alignment, 0);
        assert_eq!(struct_3.type_, raw::FFI_TYPE_STRUCT);

        assert_eq!(unsafe { (**struct_3.elements).type_ }, raw::FFI_TYPE_SINT32);
        assert_eq!(
            unsafe { (**struct_3.elements.add(1)).type_ },
            raw::FFI_TYPE_UINT32
        );
        assert_eq!(
            unsafe { (**struct_3.elements.add(2)).type_ },
            raw::FFI_TYPE_STRUCT
        );
        assert!(unsafe { (*struct_3.elements.add(3)).is_null() });

        // Fourth struct: struct
        let struct_4 = unsafe { &**(struct_3.elements.add(2)) };
        assert_eq!(struct_4.size, 0);
        assert_eq!(struct_4.alignment, 0);
        assert_eq!(struct_4.type_, raw::FFI_TYPE_STRUCT);

        assert_eq!(unsafe { (**struct_4.elements).type_ }, raw::FFI_TYPE_STRUCT);
        assert!(unsafe { (*struct_4.elements.add(1)).is_null() });

        // Fifth and final struct: nothing
        let struct_5 = unsafe { &**(struct_4.elements) };
        assert_eq!(struct_5.size, 0);
        assert_eq!(struct_5.alignment, 0);
        assert_eq!(struct_5.type_, raw::FFI_TYPE_STRUCT);

        assert!(unsafe { (*struct_5.elements).is_null() });
    }
}

#[cfg(miri)]
#[expect(non_upper_case_globals)]
mod miri {
    use crate::low::ffi_type;
    #[cfg(all(feature = "complex", not(windows)))]
    use crate::raw::FFI_TYPE_COMPLEX;
    use crate::raw::{
        FFI_TYPE_DOUBLE, FFI_TYPE_FLOAT, FFI_TYPE_LONGDOUBLE, FFI_TYPE_POINTER, FFI_TYPE_SINT16,
        FFI_TYPE_SINT32, FFI_TYPE_SINT64, FFI_TYPE_SINT8, FFI_TYPE_UINT16, FFI_TYPE_UINT32,
        FFI_TYPE_UINT64, FFI_TYPE_UINT8, FFI_TYPE_VOID,
    };
    use core::ffi::c_void;
    use core::mem::{align_of, size_of};
    #[cfg(all(feature = "complex", not(windows)))]
    use core::ptr::addr_of_mut;
    use core::ptr::null_mut;

    // Redefining static muts so this module can be tested with miri
    pub static mut sint8: ffi_type = ffi_type {
        size: size_of::<i8>(),
        alignment: align_of::<i8>() as u16,
        type_: FFI_TYPE_SINT8,
        elements: null_mut(),
    };

    pub static mut uint8: ffi_type = ffi_type {
        size: size_of::<u8>(),
        alignment: align_of::<u8>() as u16,
        type_: FFI_TYPE_UINT8,
        elements: null_mut(),
    };

    pub static mut sint16: ffi_type = ffi_type {
        size: size_of::<i16>(),
        alignment: align_of::<i16>() as u16,
        type_: FFI_TYPE_SINT16,
        elements: null_mut(),
    };

    pub static mut uint16: ffi_type = ffi_type {
        size: size_of::<u16>(),
        alignment: align_of::<u16>() as u16,
        type_: FFI_TYPE_UINT16,
        elements: null_mut(),
    };

    pub static mut sint32: ffi_type = ffi_type {
        size: size_of::<i32>(),
        alignment: align_of::<i32>() as u16,
        type_: FFI_TYPE_SINT32,
        elements: null_mut(),
    };

    pub static mut uint32: ffi_type = ffi_type {
        size: size_of::<u32>(),
        alignment: align_of::<u32>() as u16,
        type_: FFI_TYPE_UINT32,
        elements: null_mut(),
    };

    pub static mut sint64: ffi_type = ffi_type {
        size: size_of::<i64>(),
        alignment: align_of::<i64>() as u16,
        type_: FFI_TYPE_SINT64,
        elements: null_mut(),
    };

    pub static mut uint64: ffi_type = ffi_type {
        size: size_of::<u64>(),
        alignment: align_of::<u64>() as u16,
        type_: FFI_TYPE_UINT64,
        elements: null_mut(),
    };

    pub static mut pointer: ffi_type = ffi_type {
        size: size_of::<*mut c_void>(),
        alignment: align_of::<*mut c_void>() as u16,
        type_: FFI_TYPE_POINTER,
        elements: null_mut(),
    };

    pub static mut float: ffi_type = ffi_type {
        size: size_of::<f32>(),
        alignment: align_of::<f32>() as u16,
        type_: FFI_TYPE_FLOAT,
        elements: null_mut(),
    };

    pub static mut double: ffi_type = ffi_type {
        size: size_of::<f64>(),
        alignment: align_of::<f64>() as u16,
        type_: FFI_TYPE_DOUBLE,
        elements: null_mut(),
    };

    // Note that this layout is not necessarily correct and should only be used
    // to verify memory operations with miri, and not for calling foreign
    // functions.
    pub static mut longdouble: ffi_type = ffi_type {
        size: size_of::<f64>(),
        alignment: align_of::<f64>() as u16,
        type_: FFI_TYPE_LONGDOUBLE,
        elements: null_mut(),
    };

    #[cfg(all(feature = "complex", not(windows)))]
    static mut complex_float_elements: [*mut ffi_type; 2] = [addr_of_mut!(float), null_mut()];

    // Note that this layout is not necessarily correct and should only be used
    // to verify memory operations with miri, and not for calling foreign
    // functions.
    #[cfg(all(feature = "complex", not(windows)))]
    pub static mut complex_float: ffi_type = ffi_type {
        size: 2 * size_of::<f32>(),
        alignment: align_of::<f32>() as u16,
        type_: FFI_TYPE_COMPLEX,
        elements: addr_of_mut!(complex_float_elements).cast(),
    };

    #[cfg(all(feature = "complex", not(windows)))]
    static mut complex_double_elements: [*mut ffi_type; 2] = [addr_of_mut!(double), null_mut()];

    // Note that this layout is not necessarily correct and should only be used
    // to verify memory operations with miri, and not for calling foreign
    // functions.
    #[cfg(all(feature = "complex", not(windows)))]
    pub static mut complex_double: ffi_type = ffi_type {
        size: 2 * size_of::<f64>(),
        alignment: align_of::<f64>() as u16,
        type_: FFI_TYPE_COMPLEX,
        elements: addr_of_mut!(complex_double_elements).cast(),
    };

    #[cfg(all(feature = "complex", not(windows)))]
    static mut complex_longdouble_elements: [*mut ffi_type; 2] = [addr_of_mut!(double), null_mut()];

    // Note that this layout is not necessarily correct and should only be used
    // to verify memory operations with miri, and not for calling foreign
    // functions.
    #[cfg(all(feature = "complex", not(windows)))]
    pub static mut complex_longdouble: ffi_type = ffi_type {
        size: 2 * size_of::<f64>(),
        alignment: align_of::<f64>() as u16,
        type_: FFI_TYPE_COMPLEX,
        elements: addr_of_mut!(complex_longdouble_elements).cast(),
    };

    pub static mut void: ffi_type = ffi_type {
        size: 0,
        alignment: 0,
        type_: FFI_TYPE_VOID,
        elements: null_mut(),
    };
}
