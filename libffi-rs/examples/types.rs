use libffi::middle::Type;

fn main() {
    Type::structure(vec![Type::U16, Type::U16]);
}
