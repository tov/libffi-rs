use std::process::Command;

extern crate pkg_config;

const C_IN:   &'static str = "include/include_ffi.h";
const RS_OUT: &'static str = "src/raw.rs";

fn main() {
    let libffi = pkg_config::probe_library("libffi").expect("libffi");

    let mut command = Command::new("bindgen");
    command.arg(format!("--output={}", RS_OUT))
           .arg(C_IN)
           .arg("--");
    for path in &libffi.include_paths {
        command.arg(format!("-I{}", path.display()));
    }

    let status = command.status()
        .expect("Could not run bindgen. Do you have it installed?");
    assert!(status.success());

    for lib in &libffi.libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    for path in &libffi.link_paths {
        println!("cargo:rustc-link-search={}", path.display());
    }
}

