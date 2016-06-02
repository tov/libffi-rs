extern crate libffi;

use libffi::ffi_type::FfiType;

fn main() {
    FfiType::structure(vec![
        FfiType::uint16(),
        FfiType::uint16(),
    ]);
}
