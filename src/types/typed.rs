use std::marker::PhantomData;

use types as untyped;

#[derive(Clone, Debug)]
pub struct Type<T> {
    untyped: untyped::Type,
    phantom: PhantomData<T>,
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

macro_rules! declare_type_array {
    ( $typename:ident<$( $param:ident ),*> ) => {
        pub struct $typename<$( $param ),*> {
            untyped: untyped::TypeArray,
            phantom: PhantomData<($( $param, )*)>,
        }
    }
}

declare_type_array!(TypeArray1<A>);
declare_type_array!(TypeArray2<A, B>);
declare_type_array!(TypeArray3<A, B, C>);
declare_type_array!(TypeArray4<A, B, C, D>);
declare_type_array!(TypeArray5<A, B, C, D, E>);
declare_type_array!(TypeArray6<A, B, C, D, E, F>);
declare_type_array!(TypeArray7<A, B, C, D, E, F, G>);
declare_type_array!(TypeArray8<A, B, C, D, E, F, G, H>);
declare_type_array!(TypeArray9<A, B, C, D, E, F, G, H, I>);
declare_type_array!(TypeArray10<A, B, C, D, E, F, G, H, I, J>);

// This is a fun idea, but it won’t actually work unless tuples are laid
// out the same as C structs, which seems unlikely.
macro_rules! impl_ffi_type_tuple {
    ( $( $param:ident ),* ) => {
        impl<$( $param: FfiType ),*> FfiType for ($( $param, )*) {
            fn get_type() -> Type<Self> {
                let params = vec![ $( $param::get_type().untyped ),* ];
                Type::make(untyped::Type::structure(params))
            }
        }
    }
}
impl_ffi_type_tuple!(A);
impl_ffi_type_tuple!(A, B);
impl_ffi_type_tuple!(A, B, C);
impl_ffi_type_tuple!(A, B, C, D);
impl_ffi_type_tuple!(A, B, C, D, E);
impl_ffi_type_tuple!(A, B, C, D, E, F);
impl_ffi_type_tuple!(A, B, C, D, E, F, G);
impl_ffi_type_tuple!(A, B, C, D, E, F, G, H);
impl_ffi_type_tuple!(A, B, C, D, E, F, G, H, I);
impl_ffi_type_tuple!(A, B, C, D, E, F, G, H, I, J);
