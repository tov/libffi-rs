use std::marker::PhantomData;

use middle;

pub mod types;
use self::types::*;

macro_rules! declare_cif {
    ( $typename:ident $( $param:ident )*) => {
        pub struct $typename<$( $param, )* R> {
            untyped: middle::Cif,
            _marker: PhantomData<fn($( $param, )*) -> R>,
        }

        impl<$( $param, )* R> $typename<$( $param, )* R> {
            #[allow(non_snake_case)]
            pub fn new($( $param: Type<$param>, )* result: Type<R>) -> Self {
                let cif = middle::Cif::new(vec![$( $param.into_untyped() ),*],
                                           result.into_untyped());
                $typename { untyped: cif, _marker: PhantomData }
            }
        }
    }
}

declare_cif!(Cif0);
declare_cif!(Cif1 A);
declare_cif!(Cif2 A B);
declare_cif!(Cif3 A B C);
declare_cif!(Cif4 A B C D);
declare_cif!(Cif5 A B C D E);
declare_cif!(Cif6 A B C D E F);
declare_cif!(Cif7 A B C D E F G);
declare_cif!(Cif8 A B C D E F G H);
declare_cif!(Cif9 A B C D E F G H I);
declare_cif!(Cif10 A B C D E F G H I J);

macro_rules! declare_callback {
    ( $typename:ident $( $param:ident )* ) => {
        pub type $typename<U, $( $param, )* R>
            = extern "C" fn(cif:      &::low::ffi_cif,
                            result:   &mut R,
                            args:     &($( &$param, )*),
                            userdata: &U);
    }
}

declare_callback!( Callback0 );
declare_callback!( Callback1 A );
declare_callback!( Callback2 A B );
declare_callback!( Callback3 A B C );
declare_callback!( Callback4 A B C D );
declare_callback!( Callback5 A B C D E );
declare_callback!( Callback6 A B C D E F );
declare_callback!( Callback7 A B C D E F G );
declare_callback!( Callback8 A B C D E F G H );
declare_callback!( Callback9 A B C D E F G H I );
declare_callback!( Callback10 A B C D E F G H I J );

macro_rules! declare_closure {
    ( $closure:ident $cif:ident $callback:ident $( $param:ident )*) => {
        pub struct $closure<$( $param, )* R> {
            untyped: middle::Closure,
            _marker: PhantomData<fn($( $param, )*) -> R>,
        }

        impl<$( $param, )* R> $closure<$( $param, )* R> {
            pub fn new<U>(cif: $cif<$( $param, )* R>,
                          callback: $callback<U, $( $param, )* R>,
                          userdata: &mut U) -> Self
            {
                let callback: middle::Callback<U, R>
                    = unsafe { ::std::mem::transmute(callback) };
                let closure
                    = middle::Closure::new(cif.untyped,
                                           callback,
                                           userdata);
                $closure {
                    untyped: closure,
                    _marker: PhantomData,
                }
            }

            pub fn code_ptr(&self) -> extern "C" fn($( $param, )*) -> R {
                unsafe {
                    ::std::mem::transmute(self.untyped.code_ptr())
                }
            }
        }
    }
}

declare_closure!(Closure0 Cif0 Callback0);
declare_closure!(Closure1 Cif1 Callback1 A);
declare_closure!(Closure2 Cif2 Callback2 A B);
declare_closure!(Closure3 Cif3 Callback3 A B C);
declare_closure!(Closure4 Cif4 Callback4 A B C D);
declare_closure!(Closure5 Cif5 Callback5 A B C D E);
declare_closure!(Closure6 Cif6 Callback6 A B C D E F);
declare_closure!(Closure7 Cif7 Callback7 A B C D E F G);
declare_closure!(Closure8 Cif8 Callback8 A B C D E F G H);
declare_closure!(Closure9 Cif9 Callback9 A B C D E F G H I);
declare_closure!(Closure10 Cif10 Callback10 A B C D E F G H I J);
