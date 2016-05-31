extern crate libffi;

use libffi::low::FfiType;

fn main() {
    FfiType::structure(vec![
        FfiType::uint16(),
        FfiType::uint16(),
    ]);
}
