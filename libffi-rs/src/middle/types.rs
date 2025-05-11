//! Representations of C types and arrays thereof.
//!
//! These are used to describe the types of the arguments and results of
//! functions. When we construct a [CIF](super::Cif) (“Call
//! Inter<span></span>Face”), we provide a sequence of argument types
//! and a result type, and libffi uses this to figure out how to set up
//! a call to a function with those types.

use core::fmt;
use core::mem;
use core::ptr::{addr_of_mut, null_mut};
use libc;

use crate::low;

use super::util::Unique;

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

// Internally we represent types and type arrays using raw pointers,
// since this is what libffi understands. Below we wrap them with
// types that implement Drop and Clone.

type Type_ = *mut low::ffi_type;
type TypeArray_ = *mut Type_;

// Informal indication that the object should be considered owned by
// the given reference.
type Owned<T> = T;

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
/// let my_struct = Type::structure(vec![
///     Type::u64(),
///     Type::u16(),
/// ]);
/// ```
pub struct Type(Unique<low::ffi_type>);

/// Represents a sequence of C types.
///
/// This can be used to construct a struct type or as the arguments
/// when creating a [`Cif`].
pub struct TypeArray(Unique<*mut low::ffi_type>);

impl fmt::Debug for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!("Type({:?})", *self.0))
    }
}

impl fmt::Debug for TypeArray {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!("TypeArray({:?})", *self.0))
    }
}

/// Computes the length of a raw `TypeArray_` by searching for the
/// null terminator.
unsafe fn ffi_type_array_len(mut array: TypeArray_) -> usize {
    let mut count = 0;
    while !(*array).is_null() {
        count += 1;
        array = array.offset(1);
    }
    count
}

/// Creates an empty `TypeArray_` with null terminator.
unsafe fn ffi_type_array_create_empty(len: usize) -> Owned<TypeArray_> {
    let array = libc::malloc((len + 1) * mem::size_of::<Type_>()) as TypeArray_;
    assert!(
        !array.is_null(),
        "ffi_type_array_create_empty: out of memory"
    );
    *array.add(len) = null_mut::<low::ffi_type>() as Type_;
    array
}

/// Creates a null-terminated array of Type_. Takes ownership of
/// the elements.
unsafe fn ffi_type_array_create<I>(elements: I) -> Owned<TypeArray_>
where
    I: ExactSizeIterator<Item = Type>,
{
    let size = elements.len();
    let new = ffi_type_array_create_empty(size);
    for (i, element) in elements.enumerate() {
        *new.add(i) = *element.0;
        mem::forget(element);
    }

    new
}

/// Creates a struct type from a raw array of element types.
unsafe fn ffi_type_struct_create_raw(
    elements: Owned<TypeArray_>,
    size: usize,
    alignment: u16,
) -> Owned<Type_> {
    let new = libc::malloc(mem::size_of::<low::ffi_type>()) as Type_;
    assert!(!new.is_null(), "ffi_type_struct_create_raw: out of memory");

    (*new).size = size;
    (*new).alignment = alignment;
    (*new).type_ = low::type_tag::STRUCT;
    (*new).elements = elements;

    new
}

/// Creates a struct `ffi_type` with the given elements. Takes ownership
/// of the elements.
unsafe fn ffi_type_struct_create<I>(elements: I) -> Owned<Type_>
where
    I: ExactSizeIterator<Item = Type>,
{
    ffi_type_struct_create_raw(ffi_type_array_create(elements), 0, 0)
}

/// Makes a copy of a type array.
unsafe fn ffi_type_array_clone(old: TypeArray_) -> Owned<TypeArray_> {
    let size = ffi_type_array_len(old);
    let new = ffi_type_array_create_empty(size);

    for i in 0..size {
        *new.add(i) = ffi_type_clone(*old.add(i));
    }

    new
}

/// Makes a copy of a type.
unsafe fn ffi_type_clone(old: Type_) -> Owned<Type_> {
    if (*old).type_ == low::type_tag::STRUCT {
        let low::ffi_type {
            alignment,
            elements,
            size,
            ..
        } = *old;
        // Create new
        ffi_type_struct_create_raw(ffi_type_array_clone(elements), size, alignment)
    } else {
        old
    }
}

/// Destroys a `TypeArray_` and all of its elements.
unsafe fn ffi_type_array_destroy(victim: Owned<TypeArray_>) {
    let mut current = victim;
    while !(*current).is_null() {
        ffi_type_destroy(*current);
        current = current.offset(1);
    }

    libc::free(victim as *mut libc::c_void);
}

/// Destroys a `Type_` if it was dynamically allocated.
unsafe fn ffi_type_destroy(victim: Owned<Type_>) {
    if (*victim).type_ == low::type_tag::STRUCT {
        ffi_type_array_destroy((*victim).elements);
        libc::free(victim as *mut libc::c_void);
    }
}

impl Drop for Type {
    fn drop(&mut self) {
        unsafe { ffi_type_destroy(*self.0) }
    }
}

impl Drop for TypeArray {
    fn drop(&mut self) {
        unsafe { ffi_type_array_destroy(*self.0) }
    }
}

impl Clone for Type {
    fn clone(&self) -> Self {
        Type(unsafe { Unique::new(ffi_type_clone(*self.0)) })
    }
}

impl Clone for TypeArray {
    fn clone(&self) -> Self {
        TypeArray(unsafe { Unique::new(ffi_type_array_clone(*self.0)) })
    }
}

macro_rules! match_size_signed {
    ( $name:ident ) => {
        match mem::size_of::<core::ffi::$name>() {
            1 => Self::i8(),
            2 => Self::i16(),
            4 => Self::i32(),
            8 => Self::i64(),
            _ => panic!("Unsupported integer size"),
        }
    };
}

macro_rules! match_size_unsigned {
    ( $name:ident ) => {
        match mem::size_of::<core::ffi::$name>() {
            1 => Self::u8(),
            2 => Self::u16(),
            4 => Self::u32(),
            8 => Self::u64(),
            _ => panic!("Unsupported integer size"),
        }
    };
}

impl Type {
    /// Returns the representation of the C `void` type.
    ///
    /// This is used only for the return type of a [CIF](super::Cif),
    /// not for an argument or struct member.
    pub fn void() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(void)) })
    }

    /// Returns the unsigned 8-bit numeric type.
    pub fn u8() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(uint8)) })
    }

    /// Returns the signed 8-bit numeric type.
    pub fn i8() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(sint8)) })
    }

    /// Returns the unsigned 16-bit numeric type.
    pub fn u16() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(uint16)) })
    }

    /// Returns the signed 16-bit numeric type.
    pub fn i16() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(sint16)) })
    }

    /// Returns the unsigned 32-bit numeric type.
    pub fn u32() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(uint32)) })
    }

    /// Returns the signed 32-bit numeric type.
    pub fn i32() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(sint32)) })
    }

    /// Returns the unsigned 64-bit numeric type.
    pub fn u64() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(uint64)) })
    }

    /// Returns the signed 64-bit numeric type.
    pub fn i64() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(sint64)) })
    }

    #[cfg(target_pointer_width = "16")]
    /// Returns the C equivalent of Rust `usize` (`u16`).
    pub fn usize() -> Self {
        Self::u16()
    }

    #[cfg(target_pointer_width = "16")]
    /// Returns the C equivalent of Rust `isize` (`i16`).
    pub fn isize() -> Self {
        Self::i16()
    }

    #[cfg(target_pointer_width = "32")]
    /// Returns the C equivalent of Rust `usize` (`u32`).
    pub fn usize() -> Self {
        Self::u32()
    }

    #[cfg(target_pointer_width = "32")]
    /// Returns the C equivalent of Rust `isize` (`i32`).
    pub fn isize() -> Self {
        Self::i32()
    }

    #[cfg(target_pointer_width = "64")]
    /// Returns the C equivalent of Rust `usize` (`u64`).
    pub fn usize() -> Self {
        Self::u64()
    }

    #[cfg(target_pointer_width = "64")]
    /// Returns the C equivalent of Rust `isize` (`i64`).
    pub fn isize() -> Self {
        Self::i64()
    }

    /// Returns the C `signed char` type.
    pub fn c_schar() -> Self {
        match_size_signed!(c_schar)
    }

    /// Returns the C `unsigned char` type.
    pub fn c_uchar() -> Self {
        match_size_unsigned!(c_uchar)
    }

    /// Returns the C `short` type.
    pub fn c_short() -> Self {
        match_size_signed!(c_short)
    }

    /// Returns the C `unsigned short` type.
    pub fn c_ushort() -> Self {
        match_size_unsigned!(c_ushort)
    }

    /// Returns the C `int` type.
    pub fn c_int() -> Self {
        match_size_signed!(c_int)
    }

    /// Returns the C `unsigned int` type.
    pub fn c_uint() -> Self {
        match_size_unsigned!(c_uint)
    }

    /// Returns the C `long` type.
    pub fn c_long() -> Self {
        match_size_signed!(c_long)
    }

    /// Returns the C `unsigned long` type.
    pub fn c_ulong() -> Self {
        match_size_unsigned!(c_ulong)
    }

    /// Returns the C `longlong` type.
    pub fn c_longlong() -> Self {
        match_size_signed!(c_longlong)
    }

    /// Returns the C `unsigned longlong` type.
    pub fn c_ulonglong() -> Self {
        match_size_unsigned!(c_ulonglong)
    }

    /// Returns the C `float` (32-bit floating point) type.
    pub fn f32() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(float)) })
    }

    /// Returns the C `double` (64-bit floating point) type.
    pub fn f64() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(double)) })
    }

    /// Returns the C `void*` type, for passing any kind of pointer.
    pub fn pointer() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(pointer)) })
    }

    /// Returns the C `long double` (extended-precision floating point) type.
    pub fn longdouble() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(longdouble)) })
    }

    /// Returns the C `_Complex float` type.
    ///
    /// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
    #[cfg(all(feature = "complex", not(windows)))]
    pub fn c32() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(complex_float)) })
    }

    /// Returns the C `_Complex double` type.
    ///
    /// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
    #[cfg(all(feature = "complex", not(windows)))]
    pub fn c64() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(complex_double)) })
    }

    /// Returns the C `_Complex long double` type.
    ///
    /// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
    #[cfg(all(feature = "complex", not(windows)))]
    pub fn complex_longdouble() -> Self {
        Type(unsafe { Unique::new(addr_of_mut!(complex_longdouble)) })
    }

    /// Constructs a structure type whose fields have the given types.
    pub fn structure<I>(fields: I) -> Self
    where
        I: IntoIterator<Item = Type>,
        I::IntoIter: ExactSizeIterator<Item = Type>,
    {
        Type(unsafe { Unique::new(ffi_type_struct_create(fields.into_iter())) })
    }

    /// Gets a raw pointer to the underlying [`low::ffi_type`].
    ///
    /// This method may be useful for interacting with the
    /// [`low`](crate::low) and [`raw`](crate::raw) layers.
    pub fn as_raw_ptr(&self) -> *mut low::ffi_type {
        *self.0
    }
}

impl TypeArray {
    /// Constructs an array the given `Type`s.
    pub fn new<I>(elements: I) -> Self
    where
        I: IntoIterator<Item = Type>,
        I::IntoIter: ExactSizeIterator<Item = Type>,
    {
        TypeArray(unsafe { Unique::new(ffi_type_array_create(elements.into_iter())) })
    }

    /// Gets a raw pointer to the underlying C array of
    /// [`low::ffi_type`]s.
    ///
    /// The C array is null-terminated.
    ///
    /// This method may be useful for interacting with the
    /// [`low`](crate::low) and [`raw`](crate::raw) layers.
    pub fn as_raw_ptr(&self) -> *mut *mut low::ffi_type {
        *self.0
    }
}

#[cfg(all(test, feature = "std"))]
mod test {
    use alloc::format;

    use super::*;
    use crate::raw;

    #[test]
    fn create_u64() {
        Type::u64();
    }

    #[test]
    fn clone_u64() {
        let _ = Type::u64().clone().clone();
    }

    #[test]
    fn create_struct() {
        Type::structure(alloc::vec![Type::i64(), Type::i64(), Type::u64()]);
    }

    #[test]
    fn clone_struct() {
        let _ = Type::structure(alloc::vec![Type::i64(), Type::i64(), Type::u64()])
            .clone()
            .clone();
    }

    /// Verify that [`Type`]'s `Debug` impl does not misbehave.
    #[test]
    fn verify_type_debug_behavior() {
        let ffi_type = Type::structure([
            Type::u16(),
            Type::f32(),
            Type::structure([Type::i32(), Type::structure([])]),
            Type::pointer(),
        ]);

        let _string = format!("{ffi_type:?}");
    }

    #[test]
    fn verify_raw_type_layout() {
        let ffi_struct = Type::structure([
            // First struct, containing a struct, i8, and u8
            Type::structure([
                // Second struct, containing a i16, struct, and u16
                Type::i16(),
                Type::structure([
                    // Third struct, containing a i32, u32 and struct
                    Type::i32(),
                    Type::u32(),
                    Type::structure([
                        // Fourth struct, only a struct
                        Type::structure([
                            // Fifth and final struct, no members
                        ]),
                    ]),
                ]),
                Type::u16(),
            ]),
            Type::i8(),
            Type::u8(),
            Type::f32(),
            Type::f64(),
            Type::longdouble(),
            #[cfg(all(feature = "complex", not(windows)))]
            Type::c32(),
            #[cfg(all(feature = "complex", not(windows)))]
            Type::c64(),
            #[cfg(all(feature = "complex", not(windows)))]
            Type::complex_longdouble(),
        ]);

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
    fn verify_struct_layout(ty: &Type) {
        // First struct: struct, i8, u8
        let struct_1 = unsafe { &**ty.0 };
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
        assert_eq!(unsafe { (**struct_1.elements.add(5)).type_ }, unsafe {
            longdouble.type_
        });
        #[cfg(any(not(feature = "complex"), windows))]
        assert!(unsafe { (*struct_1.elements.add(6)).is_null() });
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
        assert_eq!(
            unsafe { (**struct_1.elements.add(8)).type_ },
            raw::FFI_TYPE_COMPLEX
        );
        #[cfg(all(feature = "complex", not(windows)))]
        assert!(unsafe { (*struct_1.elements.add(9)).is_null() });

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
