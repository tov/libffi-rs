#include <ffi.h>

// libffi supports several atomic types (various ints and floats) and
// two compound types: structs and complexes. Type descriptors
// (struct ffi_type) are statically allocated for the atomic types, but
// for the compound types you need to make your own, and in order to do
// so, you need to know the values of two #defines, FFI_TYPE_STRUCT and
// FFI_TYPE_COMPLEX. The easiest way to get bindgen to make those
// available to use was to use them as values in an enum definition.

enum ffi_type_enum
{
    STRUCT  = FFI_TYPE_STRUCT,
// Not all platforms/versions support complex numbers
#ifdef FFI_TYPE_COMPLEX
    COMPLEX = FFI_TYPE_COMPLEX,
#endif
};
