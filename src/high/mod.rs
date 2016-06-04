use std::marker::PhantomData;

use middle;

/// Representations of C types for the high layer.
pub mod types;
use self::types::*;

macro_rules! define_closure_types {
    (
        $cif:ident $callback:ident $callback_mut:ident
                   $closure:ident  $closure_mut:ident ;
                   $( $param:ident )*
    )
        =>
    {
        /// Typed CIF (“Call InterFace”), which statically tracks
        /// argument and result types.
        pub struct $cif<$( $param, )* R> {
            untyped: middle::Cif,
            _marker: PhantomData<fn($( $param, )*) -> R>,
        }

        impl<$( $param, )* R> $cif<$( $param, )* R> {
            /// Creates a new statically-typed CIF from the given argument
            /// and result types.
            #[allow(non_snake_case)]
            pub fn new($( $param: Type<$param>, )* result: Type<R>) -> Self {
                let cif = middle::Cif::new(vec![$( $param.into_untyped() ),*],
                                           result.into_untyped());
                $cif { untyped: cif, _marker: PhantomData }
            }
        }

        impl<$( $param: FfiType, )* R: FfiType> $cif<$( $param, )* R> {
            /// Creates a new statically-typed CIF by reifying the
            /// argument types as `Type<T>`s.
            pub fn reify() -> Self {
                Self::new($( $param::get_type(), )* R::get_type())
            }
        }

        pub type $callback<U, $( $param, )* R>
            = extern "C" fn(cif:      &::low::ffi_cif,
                            result:   &mut R,
                            args:     &($( &$param, )*),
                            userdata: &U);

        pub type $callback_mut<U, $( $param, )* R>
            = extern "C" fn(cif:      &::low::ffi_cif,
                            result:   &mut R,
                            args:     &($( &$param, )*),
                            userdata: &mut U);

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
                Self::new_with_cif($cif::reify(), callback)
            }
        }

        pub struct $closure_mut<'a, $( $param, )* R> {
            untyped: middle::Closure<'a>,
            _marker: PhantomData<fn($( $param, )*) -> R>,
        }

        impl<'a, $( $param, )* R> $closure_mut<'a, $( $param, )* R> {
            pub fn from_parts<U>(cif:      $cif<$( $param, )* R>,
                                 callback: $callback_mut<U, $( $param, )* R>,
                                 userdata: &'a mut U) -> Self
            {
                let callback: middle::Callback<U, R>
                    = unsafe { ::std::mem::transmute(callback) };
                let closure
                    = middle::Closure::new(cif.untyped,
                                           callback,
                                           userdata);
                $closure_mut {
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

        impl<'a, $( $param: Copy, )* R> $closure_mut<'a, $( $param, )* R> {
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
            $closure_mut<'a, $($param,)* R>
        {
            pub fn new<Callback>(callback: &'a mut Callback) -> Self
                where Callback: FnMut($( $param, )*) -> R + 'a
            {
                Self::new_with_cif($cif::reify(), callback)
            }
        }
    }
}

define_closure_types!(Cif0 Callback0 CallbackMut0 Closure0 ClosureMut0;
                      );
define_closure_types!(Cif1 Callback1 CallbackMut1 Closure1 ClosureMut1;
                      A);
define_closure_types!(Cif2 Callback2 CallbackMut2 Closure2 ClosureMut2;
                      A B);
define_closure_types!(Cif3 Callback3 CallbackMut3 Closure3 ClosureMut3;
                      A B C);
define_closure_types!(Cif4 Callback4 CallbackMut4 Closure4 ClosureMut4;
                      A B C D);
define_closure_types!(Cif5 Callback5 CallbackMut5 Closure5 ClosureMut5;
                      A B C D E);
define_closure_types!(Cif6 Callback6 CallbackMut6 Closure6 ClosureMut6;
                      A B C D E F);
define_closure_types!(Cif7 Callback7 CallbackMut7 Closure7 ClosureMut7;
                      A B C D E F G);
define_closure_types!(Cif8 Callback8 CallbackMut8 Closure8 ClosureMut8;
                      A B C D E F G H);
define_closure_types!(Cif9 Callback9 CallbackMut9 Closure9 ClosureMut9;
                      A B C D E F G H I);
define_closure_types!(Cif10 Callback10 CallbackMut10 Closure10 ClosureMut10;
                      A B C D E F G H I J);
define_closure_types!(Cif11 Callback11 CallbackMut11 Closure11 ClosureMut11;
                      A B C D E F G H I J K);
define_closure_types!(Cif12 Callback12 CallbackMut12 Closure12 ClosureMut12;
                      A B C D E F G H I J K L);

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
