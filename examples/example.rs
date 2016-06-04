extern crate libffi;

use libffi::middle::types::Type;

fn main() {
    Type::structure(vec![
        Type::uint16(),
        Type::uint16(),
    ]);
}
