extern crate libffi;

use libffi::types::Type;

fn main() {
    Type::structure(vec![
        Type::uint16(),
        Type::uint16(),
    ]);
}
