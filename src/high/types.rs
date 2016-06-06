use std::marker::PhantomData;

use middle::types as untyped;

#[derive(Clone, Debug)]
pub struct Type<T> {
    untyped: untyped::Type,
    _marker: PhantomData<*mut T>,
}

impl<T> Type<T> {
    fn make(untyped: untyped::Type) -> Self {
        Type {
            untyped: untyped,
            _marker: PhantomData,
        }
    }

    pub fn into_untyped(self) -> untyped::Type {
        self.untyped
    }
}

pub trait FfiType : Sized {
    fn get_type() -> Type<Self>;
}

macro_rules! impl_ffi_type {
    ($type_:ty, $cons:ident) => {
        impl FfiType for $type_ {
            fn get_type() -> Type<Self> {
                Type::make(untyped::Type::$cons())
            }
        }
    };
    ($type_:ident) => {
        impl_ffi_type!($type_, $type_);
    };
}

impl_ffi_type!(u8);
impl_ffi_type!(i8);
impl_ffi_type!(u16);
impl_ffi_type!(i16);
impl_ffi_type!(u32);
impl_ffi_type!(i32);
impl_ffi_type!(u64);
impl_ffi_type!(i64);
impl_ffi_type!(f32);
impl_ffi_type!(f64);
impl_ffi_type!(usize);
impl_ffi_type!(isize);
impl_ffi_type!((), void);

/// Laid out the same as C11 `float complex` and C++11
/// `std::complex<float>`.
///
/// # Warning
///
/// This type does not obey the ABI, and as such should not be passed by
/// value to or from a C or C++ function. Passing it via a pointer or
/// via libffi-rs is okay.
#[allow(non_camel_case_types)]
pub type c_c32 = [f32; 2];

/// Laid out the same as C11 `double complex` and C++11
/// `std::complex<double>`.
///
/// # Warning
///
/// This type does not obey the ABI, and as such should not be passed by
/// value to or from a C or C++ function. Passing it via a pointer or
/// via libffi-rs is okay.
#[allow(non_camel_case_types)]
pub type c_c64 = [f64; 2];

impl_ffi_type!(c_c32, c32);
impl_ffi_type!(c_c64, c64);

impl<T> FfiType for *const T {
    fn get_type() -> Type<Self> { Type::make(untyped::Type::pointer()) }
}

impl<T> FfiType for *mut T {
    fn get_type() -> Type<Self> { Type::make(untyped::Type::pointer()) }
}
