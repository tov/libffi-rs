#[allow(non_camel_case_types)]
pub mod bindgen;

#[derive(Debug)]
pub struct Cif(bindgen::ffi_cif);

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
    fn as_ffi_type(self) -> *mut bindgen::ffi_type {
        unsafe {
            match self {
                Type::Void => &mut bindgen::ffi_type_void,
                Type::UInt8 => &mut bindgen::ffi_type_uint8,
                Type::SInt8 => &mut bindgen::ffi_type_sint8,
                Type::UInt16 => &mut bindgen::ffi_type_uint16,
                Type::SInt16 => &mut bindgen::ffi_type_sint16,
                Type::UInt32 => &mut bindgen::ffi_type_uint32,
                Type::SInt32 => &mut bindgen::ffi_type_sint32,
                Type::UInt64 => &mut bindgen::ffi_type_uint64,
                Type::SInt64 => &mut bindgen::ffi_type_sint64,
                Type::Float => &mut bindgen::ffi_type_float,
                Type::Double => &mut bindgen::ffi_type_double,
                Type::Pointer => &mut bindgen::ffi_type_pointer,
                Type::LongDouble => &mut bindgen::ffi_type_longdouble,
                Type::ComplexFloat => &mut bindgen::ffi_type_complex_float,
                Type::ComplexDouble => &mut bindgen::ffi_type_complex_double,
                Type::ComplexLongDouble =>
                    &mut bindgen::ffi_type_complex_double,
            }
        }
    }
}

pub fn arg<T>(r: &T) -> Arg {
    Arg(unsafe { ::std::mem::transmute(r as *const T) })
}

impl Cif {
    pub fn new(args: &[Type], result: Type) -> Self {
        let mut cif: bindgen::ffi_cif = Default::default();
        let mut real_args: Vec<_> =
            args.iter().map(|t| t.as_ffi_type()).collect();

        let result = unsafe {
            bindgen::ffi_prep_cif(&mut cif,
                                  bindgen::FFI_DEFAULT_ABI,
                                  args.len() as u32,
                                  result.as_ffi_type(),
                                  real_args.as_mut_ptr())
        };

        match result {
            bindgen::ffi_status::FFI_OK
                => (),
            bindgen::ffi_status::FFI_BAD_TYPEDEF
                => panic!("FFI_BAD_TYPEDEF"),
            bindgen::ffi_status::FFI_BAD_ABI
                => panic!("FFI_BAD_ABI"),
        }

        Cif(cif)
    }

    pub unsafe fn call<R>(&self, f: usize, values: &[Arg]) -> R {
        use std::mem;

        assert!(self.0.nargs as usize == values.len());

        let mut result: R = mem::zeroed();

        bindgen::ffi_call(
            mem::transmute(&self.0),
            mem::transmute(f),
            mem::transmute(&mut result),
            mem::transmute(values.as_ptr()));

        return result;
    }
}

#[cfg(test)]
mod bindgen_test {
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
