use std::process::Command;

extern crate pkg_config;

const C_IN:   &'static str = "src/c/include_ffi.h";
const RS_OUT: &'static str = "src/bindgen/libffi.rs";

// This shouldn't be necessary, but it's required on my system when using
// sh -c as a command parser.
const DYLD_LIBRARY_PATH
            : &'static str = "/Library/Developer/CommandLineTools/usr/lib";

fn main() {
    let bindgen_cmd =
        format!("{}={} bindgen $(pkg-config --cflags libffi) {} > {}",
                "DYLD_LIBRARY_PATH", DYLD_LIBRARY_PATH,
                C_IN, RS_OUT);
    assert!{
        Command::new("sh")
                .arg("-c")
                .arg(bindgen_cmd)
                .status()
                .expect("bindgen")
                .success()
    };

    let libffi = pkg_config::probe_library("libffi").expect("libffi");

    for lib in &libffi.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    for path in &libffi.link_paths {
        println!("cargo:rustc-link-search={}", path.to_str().unwrap());
    }
}

