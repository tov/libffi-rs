extern crate libc;

mod bindgen;

pub mod ffi_type;
pub mod low;

pub use bindgen::libffi as c;
pub use ffi_type::*;
use std::mem;

#[derive(Debug)]
pub struct Cif {
    cif:    c::ffi_cif,
    args:   FfiTypeArray,
    result: FfiType,
}

#[derive(Debug)]
pub struct Arg(*mut ::std::os::raw::c_void);

pub fn arg<T>(r: &T) -> Arg {
    Arg(unsafe { mem::transmute(r as *const T) })
}

impl Cif {
    pub fn new(args: FfiTypeArray, result: FfiType) -> Self {
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
            cif: cif,
            args: args,
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
}

// pub struct Closure {
//     alloc: *mut ::std::os::raw::c_void,
//     _code:  *mut ::std::os::raw::c_void,
// }

// impl Drop for Closure {
//     fn drop(&mut self) {
//         unsafe {
//             c::ffi_closure_free(self.alloc);
//         }
//     }
// }

// impl Closure {
//     pub fn new() -> Self {
//         let mut code: *mut ::std::os::raw::c_void =
//             unsafe { mem::zeroed() };

//         let alloc = unsafe {
//             c::ffi_closure_alloc(
//                 mem::size_of::<c::ffi_closure>(),
//                 &mut code)
//         };

//         assert!(alloc as usize != 0);

//         Closure {
//             alloc: alloc,
//             _code:  code,
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ffi_call() {
        use ffi_type::*;

        let args = FfiTypeArray::new(vec![FfiType::sint64(),
                                          FfiType::sint64()]);
        let cif = Cif::new(args, FfiType::sint64());
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
