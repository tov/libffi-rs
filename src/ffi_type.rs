use std::{mem, ptr};
use libc;

use c;
use low;

type FfiType_      = *mut low::ffi_type;
type FfiTypeArray_ = *mut FfiType_;

#[derive(Debug)]
pub struct FfiType(FfiType_);

#[derive(Debug)]
pub struct FfiTypeArray {
    ptr: FfiTypeArray_,
    len: usize,
}

/// Creates a null-terminated array of FfiType_. Takes ownership of
/// the elements.
unsafe fn ffi_type_array_create(elements: Vec<FfiType>) -> FfiTypeArray_ {
    let size = elements.len();
    let array = libc::malloc((size+1) * mem::size_of::<FfiType_>())
                    as FfiTypeArray_;

    for i in 0 .. size {
        *array.offset(i as isize) = elements[i].0;
    }
    *array.offset(size as isize) = ptr::null::<low::ffi_type>() as FfiType_;

    for t in elements {
        mem::forget(t);
    }

    println!("ffi_type_array_create(...) = {:?}", array);

    array
}

unsafe fn ffi_type_struct_create_raw(elements: FfiTypeArray_) -> FfiType_ {
    let new = libc::malloc(mem::size_of::<low::ffi_type>()) as FfiType_;

    (*new).size      = 0;
    (*new).alignment = 0;
    (*new).type_     = c::ffi_type_enum::STRUCT as ::libc::c_ushort;
    (*new).elements  = elements;

    println!("ffi_type_struct_create_raw({:?}) = {:?}", elements, new);

    new
}

/// Creates a struct ffi_type with the given elements. Takes ownership
/// of the elements.
unsafe fn ffi_type_struct_create(elements: Vec<FfiType>) -> FfiType_ {
    println!("ffi_type_array_create({:?})", elements);
    ffi_type_struct_create_raw(ffi_type_array_create(elements))
}

unsafe fn ffi_type_array_clone(ffi_types: FfiTypeArray_) -> FfiTypeArray_ {
    let mut current = ffi_types;
    let mut count   = 0;
    while !(*current).is_null() {
        current = current.offset(1);
        count += 1;
    }

    let new = libc::malloc((count+1) * mem::size_of::<FfiType_>())
                    as FfiTypeArray_;

    for i in 0 .. count {
        *new.offset(i as isize) = ffi_type_clone(*ffi_types.offset(i as isize));
    }
    *new.offset(count as isize) = ptr::null::<low::ffi_type>() as FfiType_;

    new
}

unsafe fn ffi_type_clone(old: FfiType_) -> FfiType_ {
    if (*old).type_ == c::ffi_type_enum::STRUCT as u16 {
        ffi_type_struct_create_raw(ffi_type_array_clone((*old).elements))
    } else {
        old
    }
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
    if (*ffi_type).type_ == c::ffi_type_enum::STRUCT as u16 {
        ffi_type_array_destroy((*ffi_type).elements);
        libc::free(ffi_type as *mut libc::c_void);
    }
}

impl Drop for FfiType {
    fn drop(&mut self) {
        unsafe { ffi_type_destroy(self.0) }
    }
}

impl Drop for FfiTypeArray {
    fn drop(&mut self) {
        unsafe { ffi_type_array_destroy(self.ptr) }
    }
}


impl FfiType {
    pub fn void() -> Self {
        FfiType(unsafe { &mut low::ffi_type_void })
    }

    pub fn uint8() -> Self {
        FfiType(unsafe { &mut low::ffi_type_uint8 })
    }

    pub fn sint8() -> Self {
        FfiType(unsafe { &mut low::ffi_type_sint8 })
    }

    pub fn uint16() -> Self {
        FfiType(unsafe { &mut low::ffi_type_uint16 })
    }

    pub fn sint16() -> Self {
        FfiType(unsafe { &mut low::ffi_type_sint16 })
    }

    pub fn uint32() -> Self {
        FfiType(unsafe { &mut low::ffi_type_uint32 })
    }

    pub fn sint32() -> Self {
        FfiType(unsafe { &mut low::ffi_type_sint32 })
    }

    pub fn uint64() -> Self {
        FfiType(unsafe { &mut low::ffi_type_uint64 })
    }

    pub fn sint64() -> Self {
        FfiType(unsafe { &mut low::ffi_type_sint64 })
    }

    pub fn float() -> Self {
        FfiType(unsafe { &mut low::ffi_type_float })
    }

    pub fn double() -> Self {
        FfiType(unsafe { &mut low::ffi_type_double })
    }

    pub fn pointer() -> Self {
        FfiType(unsafe { &mut low::ffi_type_pointer })
    }

    pub fn longdouble() -> Self {
        FfiType(unsafe { &mut low::ffi_type_longdouble })
    }

    pub fn complex_float() -> Self {
        FfiType(unsafe { &mut low::ffi_type_complex_float })
    }

    pub fn complex_double() -> Self {
        FfiType(unsafe { &mut low::ffi_type_complex_double })
    }

    pub fn complex_longdouble() -> Self {
        FfiType(unsafe { &mut low::ffi_type_complex_longdouble })
    }

    pub fn structure(fields: Vec<FfiType>) -> Self {
        println!("FfiType::structure({:?})", fields);
        unsafe {
            FfiType(ffi_type_struct_create(fields))
        }
    }

    pub fn as_raw_ptr(&self) -> *mut low::ffi_type {
        self.0
    }
}

impl FfiTypeArray {
    pub fn new(types: Vec<FfiType>) -> Self {
        let len = types.len();
        unsafe {
            FfiTypeArray {
                ptr: ffi_type_array_create(types),
                len: len,
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_raw_ptr(&self) -> *mut *mut low::ffi_type {
        self.ptr
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_uint64() {
        FfiType::uint64();
    }

    #[test]
    fn create_struct() {
        FfiType::structure(vec![FfiType::sint64(),
                                FfiType::sint64(),
                                FfiType::uint64()]);
    }

}
