
mod common;
#[cfg(target_env = "msvc")]
mod msvc;
#[cfg(not(target_env = "msvc"))]
mod not_msvc;

use common::*;
#[cfg(target_env = "msvc")]
use msvc::*;
#[cfg(not(target_env = "msvc"))]
use not_msvc::*;

fn main() {
    let include_paths = if cfg!(feature = "system") {
        probe_and_link()
    } else {
        build_and_link()
    };

    generate_bindings(&include_paths);
}

fn probe_and_link() -> IncludePaths {
    let libffi = pkg_config::probe_library("libffi").expect("
        **********
        pkg-config could not find libffi. This could be because you
        don't have pkg-config, because you don't have libffi, or because
        they don't know about each other. If you can run `pkg-config
        libffi --cflags` and get a reasonable result, please file a bug
        report.
        **********
    ");

    IncludePaths(libffi.include_paths)
}

fn generate_bindings(include_paths: &IncludePaths) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let include_file = Path::new("include").join("include_ffi.h");
    let out_file = Path::new(&out_dir).join("generated.rs");

    let mut builder = bindgen::Builder::default();

    for path in &include_paths.0 {
        builder = builder.clang_arg(format!("-I{}", path.display()));
    }

    builder
        .header(include_file.display().to_string())
        .derive_default(true)
        .blacklist_type("max_align_t")
        .generate()
        .expect("
        **********
        Bindgen generation failed. Note that Bindgen requires clang to
        be installed. See the Bindgen documentation for more information:

            https://rust-lang.github.io/rust-bindgen/

        If you believe this should have worked, please file a bug report.
        **********
        ")
        .write_to_file(out_file)
        .expect("bindgen output");
}
