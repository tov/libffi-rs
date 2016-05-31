use super::ffi::bindgen;
use std::{fmt, mem, ptr};

pub struct FfiType(*mut bindgen::ffi_type);

/// Finds the length of a null-terminated C array of pointers.
unsafe fn arrayz_len<T>(array: *mut *mut T) -> usize {
    let mut count = 0;

    for i in 0 .. {
        let element = array.offset(i);
        if (*element).is_null() {
            break;
        }
        count += 1;
    }

    return count;
}

/// Marshalls an array of ffi_type*s as a vector. Dropping the vector will
/// free the array.
unsafe fn arrayz_to_vec(array: *mut *mut bindgen::ffi_type) -> Vec<FfiType>
{
    let size = arrayz_len(array);
    mem::transmute(Vec::from_raw_parts(array, size + 1, size + 1))
}

impl Drop for FfiType {
    fn drop(&mut self) {
        if self.0.is_null() { return }

        let ffi_type = unsafe { &*self.0 };

        if ffi_type.type_ == bindgen::ffi_type_enum::STRUCT as u16 {
            unsafe {
                drop(arrayz_to_vec(ffi_type.elements));
                drop(Box::from_raw(self.0));
            }
        }
    }
}

impl fmt::Debug for FfiType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.0.is_null() {
            return formatter.write_str("null");
        }

        let ffi_type = unsafe { &*self.0 };

        if ffi_type.type_ == bindgen::ffi_type_enum::STRUCT as u16 {
            let vec  = unsafe { arrayz_to_vec(ffi_type.elements) };
            try!(vec.fmt(formatter));
            mem::forget(vec);
        } else {
            try!(ffi_type.type_.fmt(formatter));
        }

        Ok(())
    }
}

impl Clone for FfiType {
    fn clone(&self) -> Self {
        if self.0.is_null() {
            return FfiType(self.0)
        }

        let ffi_type = unsafe { &*self.0 };

        if ffi_type.type_ == bindgen::ffi_type_enum::STRUCT as u16 {
            let vec = unsafe { arrayz_to_vec(ffi_type.elements) };
            let mut copy = vec.clone();
            mem::forget(vec);
            copy.pop();
            FfiType::structure(copy)
        } else {
            FfiType(self.0)
        }
    }
}

impl FfiType {
    fn null() -> Self {
        FfiType(unsafe { mem::transmute(ptr::null::<bindgen::ffi_type>()) })
    }

    pub fn void() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_void) })
    }

    pub fn uint8() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_uint8) })
    }

    pub fn sint8() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_sint8) })
    }

    pub fn uint16() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_uint16) })
    }

    pub fn sint16() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_sint16) })
    }

    pub fn uint32() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_uint32) })
    }

    pub fn sint32() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_sint32) })
    }

    pub fn uint64() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_uint64) })
    }

    pub fn sint64() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_sint64) })
    }

    pub fn float() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_float) })
    }

    pub fn double() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_double) })
    }

    pub fn pointer() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_pointer) })
    }

    pub fn longdouble() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_longdouble) })
    }

    pub fn complex_float() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_complex_float) })
    }

    pub fn complex_double() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_complex_double) })
    }

    pub fn complex_longdouble() -> Self {
        FfiType(unsafe { (&mut bindgen::ffi_type_complex_longdouble) })
    }

    fn compound(type_: u16, mut vec: Vec<FfiType>) -> Self {
        vec.push(FfiType::null());
        vec.shrink_to_fit();
        let fields = vec.as_mut_ptr();
        mem::forget(vec);

        let mut ffi_type: Box<bindgen::ffi_type> = Default::default();
        ffi_type.type_    = type_;
        ffi_type.elements = unsafe { mem::transmute(fields) };

        FfiType(Box::into_raw(ffi_type))
    }

    pub fn structure(fields: Vec<FfiType>) -> Self {
        Self::compound(bindgen::ffi_type_enum::STRUCT as u16, fields)
    }

    // Needs more initialization:
    // pub fn complex(field: FfiType) -> Self {
    //     Self::compound(bindgen::ffi_type_enum::COMPLEX as u16, vec![field])
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_uint64() {
        let t = FfiType::uint64();
        assert_eq!("11", format!("{:?}", t));
    }

    #[test]
    fn create_struct() {
        let t = FfiType::structure(vec![FfiType::sint64()]);
        // assert_eq!("[12, null]", format!("{:?}", t));
    }

}
