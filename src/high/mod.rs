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

macro_rules! declare_callback_mut {
    ( $typename:ident $( $param:ident )* ) => {
        pub type $typename<U, $( $param, )* R>
            = extern "C" fn(cif:      &::low::ffi_cif,
                            result:   &mut R,
                            args:     &($( &$param, )*),
                            userdata: &mut U);
    }
}

declare_callback_mut!( CallbackMut0 );
declare_callback_mut!( CallbackMut1 A );
declare_callback_mut!( CallbackMut2 A B );
declare_callback_mut!( CallbackMut3 A B C );
declare_callback_mut!( CallbackMut4 A B C D );
declare_callback_mut!( CallbackMut5 A B C D E );
declare_callback_mut!( CallbackMut6 A B C D E F );
declare_callback_mut!( CallbackMut7 A B C D E F G );
declare_callback_mut!( CallbackMut8 A B C D E F G H );
declare_callback_mut!( CallbackMut9 A B C D E F G H I );
declare_callback_mut!( CallbackMut10 A B C D E F G H I J );

macro_rules! declare_closure {
    ( $closure:ident $cif:ident $callback:ident $( $param:ident )*) => {
        pub struct $closure<'a, $( $param, )* R> {
            untyped: middle::Closure<'a>,
            _marker: PhantomData<fn($( $param, )*) -> R>,
        }

        impl<'a, $( $param, )* R> $closure<'a, $( $param, )* R> {
            pub fn from_parts<U>(cif: $cif<$( $param, )* R>,
                                 callback: $callback<U, $( $param, )* R>,
                                 userdata: &'a U) -> Self
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

            pub fn code_ptr(&self) -> &extern "C" fn($( $param, )*) -> R {
                unsafe {
                    ::std::mem::transmute(self.untyped.code_ptr())
                }
            }
        }

        impl<'a, $( $param: Copy, )* R> $closure<'a, $( $param, )* R> {
            pub fn new_with_cif<Callback>(cif: $cif<$( $param, )* R>,
                                          callback: &'a Callback) -> Self
                where Callback: Fn($( $param, )*) -> R + 'a
            {
                Self::from_parts(cif,
                                 Self::static_callback,
                                 callback)
            }

            #[allow(non_snake_case)]
            extern "C" fn static_callback<Callback>
                (_cif:     &::low::ffi_cif,
                 result:   &mut R,
                 &($( &$param, )*):
                           &($( &$param, )*),
                 userdata: &Callback)
              where Callback: Fn($( $param, )*) -> R + 'a
            {
                *result = userdata($( $param, )*);
            }
        }

        impl<'a, $($param: Copy + FfiType,)* R: FfiType>
            $closure<'a, $($param,)* R>
        {
            pub fn new<Callback>(callback: &'a Callback) -> Self
                where Callback: Fn($( $param, )*) -> R + 'a
            {
                let cif = $cif::new($( $param::get_type(), )* R::get_type());
                Self::new_with_cif(cif, callback)
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

macro_rules! declare_closure_mut {
    ( $closure:ident $cif:ident $callback:ident $( $param:ident )*) => {
        pub struct $closure<'a, $( $param, )* R> {
            untyped: middle::Closure<'a>,
            _marker: PhantomData<fn($( $param, )*) -> R>,
        }

        impl<'a, $( $param, )* R> $closure<'a, $( $param, )* R> {
            pub fn from_parts<U>(cif: $cif<$( $param, )* R>,
                                 callback: $callback<U, $( $param, )* R>,
                                 userdata: &'a mut U) -> Self
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

            pub fn code_ptr(&self) -> &extern "C" fn($( $param, )*) -> R {
                unsafe {
                    ::std::mem::transmute(self.untyped.code_ptr())
                }
            }
        }

        impl<'a, $( $param: Copy, )* R> $closure<'a, $( $param, )* R> {
            pub fn new_with_cif<Callback>(cif: $cif<$( $param, )* R>,
                                          callback: &'a mut Callback) -> Self
                where Callback: FnMut($( $param, )*) -> R + 'a
            {
                Self::from_parts(cif,
                                 Self::static_callback,
                                 callback)
            }

            #[allow(non_snake_case)]
            extern "C" fn static_callback<Callback>
                (_cif:     &::low::ffi_cif,
                 result:   &mut R,
                 &($( &$param, )*):
                           &($( &$param, )*),
                 userdata: &mut Callback)
              where Callback: FnMut($( $param, )*) -> R + 'a
            {
                *result = userdata($( $param, )*);
            }
        }

        impl<'a, $($param: Copy + FfiType,)* R: FfiType>
            $closure<'a, $($param,)* R>
        {
            pub fn new<Callback>(callback: &'a mut Callback) -> Self
                where Callback: FnMut($( $param, )*) -> R + 'a
            {
                let cif = $cif::new($( $param::get_type(), )* R::get_type());
                Self::new_with_cif(cif, callback)
            }
        }
    }
}

declare_closure_mut!(ClosureMut0 Cif0 CallbackMut0);
declare_closure_mut!(ClosureMut1 Cif1 CallbackMut1 A);
declare_closure_mut!(ClosureMut2 Cif2 CallbackMut2 A B);
declare_closure_mut!(ClosureMut3 Cif3 CallbackMut3 A B C);
declare_closure_mut!(ClosureMut4 Cif4 CallbackMut4 A B C D);
declare_closure_mut!(ClosureMut5 Cif5 CallbackMut5 A B C D E);
declare_closure_mut!(ClosureMut6 Cif6 CallbackMut6 A B C D E F);
declare_closure_mut!(ClosureMut7 Cif7 CallbackMut7 A B C D E F G);
declare_closure_mut!(ClosureMut8 Cif8 CallbackMut8 A B C D E F G H);
declare_closure_mut!(ClosureMut9 Cif9 CallbackMut9 A B C D E F G H I);
declare_closure_mut!(ClosureMut10 Cif10 CallbackMut10 A B C D E F G H I J);

#[cfg(test)]
mod test {
    use super::*;
    use super::types::*;

    #[test]
    fn new_with_cif() {
        let x: u64 = 1;
        let f = |y: u64, z: u64| x + y + z;

        let type_   = u64::get_type();
        let cif     = Cif2::new(type_.clone(), type_.clone(), type_.clone());
        let closure = Closure2::new_with_cif(cif, &f);

        assert_eq!(12, closure.code_ptr()(5, 6));
    }

    #[test]
    fn new_with_cif_mut() {
        let mut x: u64 = 0;
        let mut f = |y: u64| { x += y; x };

        let type_   = u64::get_type();
        let cif     = Cif1::new(type_.clone(), type_.clone());
        let closure = ClosureMut1::new_with_cif(cif, &mut f);

        let counter = closure.code_ptr();

        assert_eq!(5, counter(5));
        assert_eq!(6, counter(1));
        assert_eq!(8, counter(2));
    }

    #[test]
    fn new() {
        let x: u64 = 1;
        let f = |y: u64, z: u64| x + y + z;

        let closure = Closure2::new(&f);

        assert_eq!(12, closure.code_ptr()(5, 6));
    }
}
