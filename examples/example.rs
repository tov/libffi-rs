extern crate libffi;

use libffi::middle::types::Type;

fn main() {
    Type::structure(vec![
        Type::u16(),
        Type::u16(),
    ]);
}
