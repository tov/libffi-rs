#[allow(non_camel_case_types)]
pub mod bindgen;

#[cfg(test)]
mod bindgen_test {
    use super::bindgen as bg;
    use std::mem;

    #[test]
    fn ffi_call() {
        let mut cif: bg::ffi_cif = Default::default();
        let mut rc: bg::ffi_arg  = Default::default();
        let mut args: Vec<*mut bg::ffi_type> =
            unsafe {
                vec![&mut bg::ffi_type_sint64]
            };
        let mut values: Vec<i64> = vec![5, 7];

        unsafe {
            bg::ffi_prep_cif(&mut cif,
                             bg::FFI_DEFAULT_ABI,
                             1,
                             &mut bg::ffi_type_sint64,
                             args.as_mut_ptr());

            bg::ffi_call(&mut cif,
                         Some(mem::transmute(add_it)),
                         mem::transmute(&mut rc),
                         mem::transmute(values.as_mut_ptr()));
        }
    }

    extern "C" fn add_it(n: i64) -> i64 {
        n + 1
    }
}
