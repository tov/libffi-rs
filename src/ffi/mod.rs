#[allow(non_camel_case_types)]
pub mod bindgen;

#[cfg(test)]
mod bindgen_test {
    use super::bindgen as bg;
    use std::mem;

    #[test]
    fn ffi_call() {
        let mut cif: bg::ffi_cif = Default::default();
        let mut rc: i64 = 0;
        let mut args: Vec<*mut bg::ffi_type> =
            vec![ unsafe {&mut bg::ffi_type_sint64}
                , unsafe {&mut bg::ffi_type_sint64} ];
        let mut values: Vec<*mut i64> =
            vec![ &mut 5i64 as *mut i64
                , &mut 7i64 as *mut i64 ];

        unsafe {
            bg::ffi_prep_cif(&mut cif,
                             bg::FFI_DEFAULT_ABI,
                             2,
                             &mut bg::ffi_type_sint64,
                             args.as_mut_ptr());

            bg::ffi_call(&mut cif,
                         mem::transmute(add_it as usize),
                         mem::transmute(&mut rc),
                         mem::transmute(values.as_mut_ptr()));
        }

        assert_eq!(12, rc);
    }

    extern "C" fn add_it(n: i64, m: i64) -> i64 {
        return n + m;
    }
}
