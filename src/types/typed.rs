use types as untyped;

#[derive(Clone, Debug)]
pub struct Type<T> {
    untyped: untyped::Type,
    phantom: ::std::marker::PhantomData<T>,
}

impl<T> Type<T> {
    fn make(untyped: untyped::Type) -> Self {
        Type {
            untyped: untyped,
            phantom: Default::default(),
        }
    }
}

pub trait FfiType : Sized {
    fn get_type() -> Type<Self>;
}

macro_rules! impl_ffi_type {
    ($type_:ident, $cons:ident) => {
        impl FfiType for $type_ {
            fn get_type() -> Type<Self> {
                Type::make(untyped::Type::$cons())
            }
        }
    }
}

impl_ffi_type!(u8, uint8);
impl_ffi_type!(i8, sint8);
impl_ffi_type!(u16, uint16);
impl_ffi_type!(i16, sint16);
impl_ffi_type!(u32, uint32);
impl_ffi_type!(i32, sint32);
impl_ffi_type!(u64, uint64);
impl_ffi_type!(i64, sint64);
impl_ffi_type!(f32, float);
impl_ffi_type!(f64, double);

impl<'a, T> FfiType for &'a T {
    fn get_type() -> Type<Self> { Type::make(untyped::Type::pointer()) }
}

impl<T> FfiType for *const T {
    fn get_type() -> Type<Self> { Type::make(untyped::Type::pointer()) }
}
