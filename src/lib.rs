extern crate libc;

mod bindgen;
pub use bindgen::libffi as c;

pub mod ffi_type;
pub mod low;

use bindgen::libffi as bg;
use std::mem;

#[derive(Debug)]
pub struct Cif(bg::ffi_cif);

#[derive(Debug)]
pub struct Arg(*mut ::std::os::raw::c_void);

#[derive(Copy, Clone, Debug)]
pub enum Type {
    Void,
    UInt8,
    SInt8,
    UInt16,
    SInt16,
    UInt32,
    SInt32,
    UInt64,
    SInt64,
    Float,
    Double,
    Pointer,
    LongDouble,
    ComplexFloat,
    ComplexDouble,
    ComplexLongDouble,
}

impl Type {
    fn as_ffi_type(self) -> *mut bg::ffi_type {
        use Type::*;
        unsafe {
            match self {
                Void => &mut bg::ffi_type_void,
                UInt8 => &mut bg::ffi_type_uint8,
                SInt8 => &mut bg::ffi_type_sint8,
                UInt16 => &mut bg::ffi_type_uint16,
                SInt16 => &mut bg::ffi_type_sint16,
                UInt32 => &mut bg::ffi_type_uint32,
                SInt32 => &mut bg::ffi_type_sint32,
                UInt64 => &mut bg::ffi_type_uint64,
                SInt64 => &mut bg::ffi_type_sint64,
                Float => &mut bg::ffi_type_float,
                Double => &mut bg::ffi_type_double,
                Pointer => &mut bg::ffi_type_pointer,
                LongDouble => &mut bg::ffi_type_longdouble,
                ComplexFloat => &mut bg::ffi_type_complex_float,
                ComplexDouble => &mut bg::ffi_type_complex_double,
                ComplexLongDouble => &mut bg::ffi_type_complex_double,
            }
        }
    }
}

pub fn arg<T>(r: &T) -> Arg {
    Arg(unsafe { mem::transmute(r as *const T) })
}

impl Cif {
    pub fn new(args: &[Type], result: Type) -> Self {
        let mut cif: bg::ffi_cif = Default::default();
        let mut real_args: Vec<_> =
            args.iter().map(|t| t.as_ffi_type()).collect();

        let result = unsafe {
            bg::ffi_prep_cif(&mut cif,
                                  bg::FFI_DEFAULT_ABI,
                                  args.len() as u32,
                                  result.as_ffi_type(),
                                  real_args.as_mut_ptr())
        };

        match result {
            bg::ffi_status::FFI_OK
                => (),
            bg::ffi_status::FFI_BAD_TYPEDEF
                => panic!("FFI_BAD_TYPEDEF"),
            bg::ffi_status::FFI_BAD_ABI
                => panic!("FFI_BAD_ABI"),
        }

        Cif(cif)
    }

    pub unsafe fn call<R>(&self, f: usize, values: &[Arg]) -> R {
        use std::mem;

        assert!(self.0.nargs as usize == values.len());

        let mut result: R = mem::zeroed();

        bg::ffi_call(
            mem::transmute(&self.0),
            mem::transmute(f),
            mem::transmute(&mut result),
            mem::transmute(values.as_ptr()));

        return result;
    }
}

pub struct Closure {
    alloc: *mut ::std::os::raw::c_void,
    _code:  *mut ::std::os::raw::c_void,
}

impl Drop for Closure {
    fn drop(&mut self) {
        unsafe {
            bg::ffi_closure_free(self.alloc);
        }
    }
}

impl Closure {
    pub fn new() -> Self {
        let mut code: *mut ::std::os::raw::c_void =
            unsafe { mem::zeroed() };

        let alloc = unsafe {
            bg::ffi_closure_alloc(
                mem::size_of::<bg::ffi_closure>(),
                &mut code)
        };

        assert!(alloc as usize != 0);

        Closure {
            alloc: alloc,
            _code:  code,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ffi_call() {
        let cif = Cif::new(&[Type::SInt64, Type::SInt64], Type::SInt64);
        let f   = |m: i64, n: i64| -> i64 {
            unsafe { cif.call(add_it as usize, &[arg(&m), arg(&n)]) }
        };

        assert_eq!(12, f(5, 7));
        assert_eq!(13, f(6, 7));
        assert_eq!(15, f(8, 7));
    }

    extern "C" fn add_it(n: i64, m: i64) -> i64 {
        return n + m;
    }
}
