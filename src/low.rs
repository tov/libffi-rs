use super::ffi::bindgen;
use std::{mem, ptr};
use libc;

type FfiType_      = *mut bindgen::ffi_type;
type FfiTypeArray_ = *mut FfiType_;

#[derive(Debug)]
pub struct FfiType(FfiType_);

#[derive(Debug)]
pub struct FfiTypeArray(FfiTypeArray_);

// /// Finds the length of a null-terminated C array of pointers.
// unsafe fn arrayz_len<T>(array: *mut *mut T) -> usize {
//     let mut count = 0;

//     for i in 0 .. {
//         let element = array.offset(i);
//         if (*element).is_null() {
//             break;
//         }
//         count += 1;
//     }

//     return count;
// }

/// Creates a null-terminated array of FfiType_. Takes ownership of
/// the elements.
unsafe fn ffi_type_array_create(elements: &[FfiType_]) -> FfiTypeArray_ {
    let size = elements.len();
    let array = libc::malloc((size+1) * mem::size_of::<FfiType_>())
                    as FfiTypeArray_;

    for i in 0 .. size {
        *array.offset(i as isize) = elements[i];
    }
    *array.offset(size as isize) = ptr::null::<bindgen::ffi_type>() as FfiType_;

    println!("ffi_type_array_create({:?}) = {:?}", elements, array);

    array
}

/// Creates a struct ffi_type with the given elements. Takes ownership
/// of the elements.
unsafe fn ffi_type_struct_create(elements: &[FfiType_]) -> FfiType_ {
    println!("ffi_type_array_create({:?})", elements);
    let array    = ffi_type_array_create(elements);
    let ffi_type = libc::malloc(mem::size_of::<bindgen::ffi_type>())
                       as FfiType_;

    (*ffi_type).size      = 0;
    (*ffi_type).alignment = 0;
    (*ffi_type).type_     = bindgen::ffi_type_enum::STRUCT as u16;
    (*ffi_type).elements  = array;

    println!("ffi_type_array_create({:?}) = {:?}", elements, ffi_type);

    ffi_type
}

/// Destroys an array of FfiType_ and all of its elements.
unsafe fn ffi_type_array_destroy(ffi_types: FfiTypeArray_) {
    println!("ffi_type_array_destroy({:?})", ffi_types);
    let mut current = ffi_types;
    while !(*current).is_null() {
        ffi_type_destroy(*current);
        current = current.offset(1);
    }

    libc::free(ffi_types as *mut libc::c_void);
}

/// Destroys an FfiType_ if it was dynamically allocated.
unsafe fn ffi_type_destroy(ffi_type: FfiType_) {
    println!("ffi_type_destroy({:?})", ffi_type);
    if ffi_type.is_null() { return }

    let type_ = (*ffi_type).type_;

    if type_ == bindgen::ffi_type_enum::STRUCT as u16 {
        ffi_type_array_destroy((*ffi_type).elements);
        libc::free(ffi_type as *mut libc::c_void);
    }
}

// /// Marshalls an array of ffi_type*s as a vector. Dropping the vector will
// /// free the array.
// unsafe fn arrayz_to_vec(array: *mut *mut bindgen::ffi_type) -> Vec<FfiType>
// {
//     let size = arrayz_len(array);
//     mem::transmute(Vec::from_raw_parts(array, size + 1, size + 1))
// }

impl Drop for FfiType {
    fn drop(&mut self) {
        unsafe { ffi_type_destroy(self.0) }
    }
}

// impl fmt::Debug for FfiType {
//     fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         if self.0.is_null() {
//             return formatter.write_str("null");
//         }

//         let ffi_type = unsafe { &*self.0 };

//         if ffi_type.type_ == bindgen::ffi_type_enum::STRUCT as u16 {
//             let vec  = unsafe { arrayz_to_vec(ffi_type.elements) };
//             try!(vec.fmt(formatter));
//             mem::forget(vec);
//         } else {
//             try!(ffi_type.type_.fmt(formatter));
//         }

//         Ok(())
//     }
// }

// impl Clone for FfiType {
//     fn clone(&self) -> Self {
//         if self.0.is_null() {
//             return FfiType(self.0)
//         }

//         let ffi_type = unsafe { &*self.0 };

//         if ffi_type.type_ == bindgen::ffi_type_enum::STRUCT as u16 {
//             let vec = unsafe { arrayz_to_vec(ffi_type.elements) };
//             let mut copy = vec.clone();
//             mem::forget(vec);
//             copy.pop();
//             FfiType::structure(copy)
//         } else {
//             FfiType(self.0)
//         }
//     }
// }

impl FfiType {
    pub fn void() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_void })
    }

    pub fn uint8() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_uint8 })
    }

    pub fn sint8() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_sint8 })
    }

    pub fn uint16() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_uint16 })
    }

    pub fn sint16() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_sint16 })
    }

    pub fn uint32() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_uint32 })
    }

    pub fn sint32() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_sint32 })
    }

    pub fn uint64() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_uint64 })
    }

    pub fn sint64() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_sint64 })
    }

    pub fn float() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_float })
    }

    pub fn double() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_double })
    }

    pub fn pointer() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_pointer })
    }

    pub fn longdouble() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_longdouble })
    }

    pub fn complex_float() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_complex_float })
    }

    pub fn complex_double() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_complex_double })
    }

    pub fn complex_longdouble() -> Self {
        FfiType(unsafe { &mut bindgen::ffi_type_complex_longdouble })
    }

    pub fn structure(fields: Vec<FfiType>) -> Self {
        println!("FfiType::structure({:?})", fields);
        let fields: Vec<FfiType_> = fields.into_iter().map(|t| t.0).collect();
        unsafe {
            FfiType(ffi_type_struct_create(fields.as_slice()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_uint64() {
        let t = FfiType::uint64();
        // assert_eq!("11", format!("{:?}", t));
    }

    #[test]
    fn create_struct() {
        let t = FfiType::structure(vec![FfiType::sint64(),
                                        FfiType::sint64(),
                                        FfiType::uint64()]);
        // assert_eq!("[12, null]", format!("{:?}", t));
        ::std::mem::forget(t);
    }

}
