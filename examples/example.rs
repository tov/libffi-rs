extern crate libffi;

use libffi::types::FfiType;

fn main() {
    FfiType::structure(vec![
        FfiType::uint16(),
        FfiType::uint16(),
    ]);
}
