extern crate libc;

mod bindgen;

pub mod types;
pub mod low;

pub use bindgen::libffi as c;
pub use types::*;

use std::mem;
use std::os::raw::c_void;

#[derive(Clone, Debug)]
pub struct Cif {
    cif:    c::ffi_cif,
    args:   FfiTypeArray,
    result: FfiType,
}

#[derive(Debug)]
pub struct Arg(*mut c_void);

pub fn arg<T>(r: &T) -> Arg {
    Arg(unsafe { mem::transmute(r as *const T) })
}

impl Cif {
    pub fn new(args: Vec<FfiType>, result: FfiType) -> Self {
        Self::from_type_array(FfiTypeArray::new(args), result)
    }

    pub fn from_type_array(args: FfiTypeArray, result: FfiType) -> Self {
        let mut cif: c::ffi_cif = Default::default();

        unsafe {
            low::prep_cif(&mut cif,
                          c::FFI_DEFAULT_ABI,
                          args.len(),
                          result.as_raw_ptr(),
                          args.as_raw_ptr())
        }.expect("low::prep_cif");

        // Note that cif retains references to args and result,
        // which is why we hold onto them here.
        Cif {
            cif:    cif,
            args:   args,
            result: result,
        }
    }

    pub unsafe fn call<R>(&self, f: usize, values: &[Arg]) -> R {
        use std::mem;

        assert!(self.cif.nargs as usize == values.len());

        low::call::<R>(mem::transmute(&self.cif),
                       mem::transmute(f),
                       mem::transmute(values.as_ptr()))
    }

    pub fn as_raw_ptr(&self) -> *mut c::ffi_cif {
        unsafe { mem::transmute(&self.cif) }
    }
}

pub struct Closure {
    _cif:  Box<Cif>,
    alloc: *mut ::low::ffi_closure,
    code:  extern "C" fn(),
}

impl Drop for Closure {
    fn drop(&mut self) {
        unsafe {
            low::closure_free(self.alloc);
        }
    }
}

impl Closure {
    pub fn new<U, R>(cif:  Cif,
                     callback: low::Callback<U, R>,
                     userdata: *mut U) -> Self
    {
        let cif = Box::new(cif);
        let (alloc, code) = low::closure_alloc();

        unsafe {
            low::prep_closure_loc(alloc,
                                  cif.as_raw_ptr(),
                                  callback,
                                  userdata,
                                  code).unwrap();
        }

        Closure {
            _cif:  cif,
            alloc: alloc,
            code:  code,
        }
    }

    pub fn code_ptr(&self) -> unsafe extern "C" fn() {
        self.code
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem;
    use std::os::raw::c_void;

    #[test]
    fn call() {
        use types::*;

        let args = vec![FfiType::sint64(),
                        FfiType::sint64()];
        let cif  = Cif::new(args, FfiType::sint64());
        let f    = |m: i64, n: i64| -> i64 {
            unsafe { cif.call(add_it as usize, &[arg(&m), arg(&n)]) }
        };

        assert_eq!(12, f(5, 7));
        assert_eq!(13, f(6, 7));
        assert_eq!(15, f(8, 7));
    }

    extern "C" fn add_it(n: i64, m: i64) -> i64 {
        return n + m;
    }

    #[test]
    fn closure() {
        use types::*;
        let cif  = Cif::new(vec![FfiType::uint64()], FfiType::uint64());
        let mut env: u64 = 5;

        unsafe {
            let closure = Closure::new(cif.clone(),
                                       callback,
                                       &mut env);
            let fun: unsafe extern "C" fn(u64) -> u64
                = mem::transmute(closure.code_ptr());

            assert_eq!(11, fun(6));
            assert_eq!(12, fun(7));
        }
    }

    unsafe extern "C" fn callback(_cif: &low::ffi_cif,
                                  result: &mut u64,
                                  args: *const *const c_void,
                                  userdata: &u64)
    {
        let args: *const &u64 = mem::transmute(args);
        *result = **args + *userdata;
    }
}
