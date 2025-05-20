//! Representations of C types for the high layer.

use core::marker::PhantomData;

use crate::middle;

/// Represents a C type statically associated with a Rust type.
///
/// In particular, the run-time value describes a particular C type,
/// while the type parameter `T` is the equivalent Rust type.
/// Instances of this type are created via the [`CType`] trait.
#[derive(Clone, Debug)]
pub struct Type<T> {
    untyped: middle::Type,
    _marker: PhantomData<*mut T>,
}

impl<T> Type<T> {
    /// Make a high type from a middle type.
    pub fn make(untyped: middle::Type) -> Self {
        Type {
            untyped,
            _marker: PhantomData,
        }
    }

    /// Gets the underlying representation as used by the
    /// [`mod@middle`] layer.
    pub fn into_middle(self) -> middle::Type {
        self.untyped
    }
}

/// Types that we can automatically marshall to/from C.
///
/// In particular, for any type `T` that implements `CType`, we can
/// get a `Type<T>` for describing that type.
///
/// # Safety
///
/// This trait is unsafe to implement because if the libffi type
/// associated with a Rust type doesn’t match then we get
/// undefined behavior.
///
/// # Examples
///
/// ```
/// use libffi::high::{CType, Type};
/// use libffi::middle;
///
/// #[derive(Clone, Copy)]
/// #[repr(transparent)]
/// struct U32Struct(u32);
///
/// unsafe impl CType for U32Struct {
///     fn reify() -> Type<Self> {
///         Type::make(middle::Type::U32)
///     }
/// }
///
/// #[derive(Clone, Copy)]
/// #[repr(C)]
/// struct F32Vec2 {
///     x: f32,
///     y: f32
/// }
///
/// unsafe impl CType for F32Vec2 {
///     fn reify() -> Type<Self> {
///         Type::make(middle::Type::structure([
///             middle::Type::F32,
///             middle::Type::F32,
///         ]))
///     }
/// }
/// ```
pub unsafe trait CType: Copy {
    /// Creates or retrieves a `Type<T>` for any type `T: CType`.
    ///
    /// We can use the resulting object to assemble a CIF to set up
    /// a call that uses type `T`.
    fn reify() -> Type<Self>;
}

macro_rules! impl_ffi_type {
    ($type_:ty, $cons:ident) => {
        unsafe impl CType for $type_ {
            fn reify() -> Type<Self> {
                Type::make(middle::Type::$cons)
            }
        }
    };
}

// We assume that `ffi_arg` and `ffi_sarg` are either 32-bit or 64-bit
// integer types on all supported platforms here.
impl_ffi_type!(u8, U8);
impl_ffi_type!(i8, I8);
impl_ffi_type!(u16, U16);
impl_ffi_type!(i16, I16);
impl_ffi_type!(u32, U32);
impl_ffi_type!(i32, I32);
impl_ffi_type!(u64, U64);
impl_ffi_type!(i64, I64);
impl_ffi_type!(f32, F32);
impl_ffi_type!(f64, F64);
impl_ffi_type!(usize, Usize);
impl_ffi_type!(isize, Isize);

// Why is the complex stuff even here? It doesn’t work yet because
// libffi doesn’t support it, so it should probably go away and come
// back when it’s actually useful. Also, the definitions for c_c32 and
// c_c64 should come from elsewhere (the num package?), but that
// elsewhere doesn’t seem to exist yet.

/// Laid out the same as C11 `float complex` and C++11
/// `std::complex<float>`.
///
/// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
///
/// # Warning
///
/// This type does not obey the ABI, and as such should not be passed by
/// value to or from a C or C++ function. Passing it via a pointer is
/// okay. Theoretically, passing it via libffi is okay, but libffi
/// doesn’t have complex support on most platforms yet.
#[allow(non_camel_case_types)]
#[cfg(all(feature = "complex", not(windows)))]
pub type c_c32 = [f32; 2];

/// Laid out the same as C11 `double complex` and C++11
/// `std::complex<double>`.
///
/// This item is enabled by `#[cfg(all(feature = "complex", not(windows)))]`.
///
/// # Warning
///
/// This type does not obey the ABI, and as such should not be passed by
/// value to or from a C or C++ function. Passing it via a pointer is
/// okay. Theoretically, passing it via libffi is okay, but libffi
/// doesn’t have complex support on most platforms yet.
#[allow(non_camel_case_types)]
#[cfg(all(feature = "complex", not(windows)))]
pub type c_c64 = [f64; 2];

#[cfg(all(feature = "complex", not(windows)))]
impl_ffi_type!(c_c32, ComplexFloat);

#[cfg(all(feature = "complex", not(windows)))]
impl_ffi_type!(c_c64, ComplexDouble);

unsafe impl<T> CType for *const T {
    fn reify() -> Type<Self> {
        Type::make(middle::Type::Pointer)
    }
}

unsafe impl<T> CType for *mut T {
    fn reify() -> Type<Self> {
        Type::make(middle::Type::Pointer)
    }
}

mod private {
    pub trait Sealed {}
}

/// Trait implemented by all types that can be returned from a function called
/// through libffi.
///
/// This trait is implemented for all types that implement `CType`, in addition
/// to `()` for functions that do not return a value. It cannot be implemented
/// manually.
///
/// # Safety
///
/// This trait is unsafe for the same reasons that [`CType`] is, as it describes
/// how libffi will read the return value. A mismatch in the layout of the type
/// implementing `CRetType` and the signature libffi expects will lead to
/// undefined behavior.
pub unsafe trait CRetType: Copy + private::Sealed {
    /// Used to describe the return type (if any) of functions that return
    /// a value of the type implementing this trait.
    fn get_return_type() -> Option<middle::Type>;
}

impl<T> private::Sealed for T where T: CType {}

unsafe impl<T> CRetType for T
where
    T: CType,
{
    fn get_return_type() -> Option<middle::Type> {
        Some(<T as CType>::reify().into_middle())
    }
}

impl private::Sealed for () {}

unsafe impl CRetType for () {
    fn get_return_type() -> Option<middle::Type> {
        None
    }
}
