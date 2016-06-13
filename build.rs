extern crate bindgen;
extern crate pkg_config;

const C_IN:   &'static str = "include/include_ffi.h";
const RS_OUT: &'static str = "src/raw.rs";

fn main() {
    let libffi = pkg_config::probe_library("libffi").expect("libffi");

    let mut builder = bindgen::Builder::default();
    builder.header(C_IN);
    for path in &libffi.include_paths {
        builder.clang_arg(format!("-I{}", path.display()));
    }

    let bindings = builder.generate().expect("bindgen generation");
    bindings.write_to_file(RS_OUT).expect("bindgen output");

    for lib in &libffi.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    for path in &libffi.link_paths {
        println!("cargo:rustc-link-search={}", path.display());
    }
}

