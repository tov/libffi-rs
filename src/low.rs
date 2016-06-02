use super::bindgen::libffi as bg;

pub use bg::ffi_abi;
pub use bg::_ffi_type as ffi_type;
pub use bg::ffi_status;

pub use bg::ffi_cif;
pub use bg::ffi_closure;

pub use bg::ffi_type_void;
pub use bg::ffi_type_uint8;
pub use bg::ffi_type_sint8;
pub use bg::ffi_type_uint16;
pub use bg::ffi_type_sint16;
pub use bg::ffi_type_uint32;
pub use bg::ffi_type_sint32;
pub use bg::ffi_type_uint64;
pub use bg::ffi_type_sint64;
pub use bg::ffi_type_float;
pub use bg::ffi_type_double;
pub use bg::ffi_type_pointer;
pub use bg::ffi_type_longdouble;
pub use bg::ffi_type_complex_float;
pub use bg::ffi_type_complex_double;
pub use bg::ffi_type_complex_longdouble;
