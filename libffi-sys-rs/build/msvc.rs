use crate::common::*;

const INCLUDE_DIRS: &[&str] = &["libffi", "libffi/include", "include/msvc", "libffi/src/x86"];

const BUILD_FILES: &[&str] = &[
    "tramp.c",
    "closures.c",
    "prep_cif.c",
    "raw_api.c",
    "types.c",
    "x86/ffi.c",
];

const BUILD_FILES_X64: &[&str] = &["x86/ffiw64.c"];

fn add_file(build: &mut cc::Build, file: &str) {
    build.file(format!("libffi/src/{}", file));
}

pub fn build_and_link() {
    let target = env::var("TARGET").unwrap();
    let is_x64 = target.contains("x86_64");
    let asm_path = pre_process_asm(INCLUDE_DIRS, &target, is_x64);
    let mut build = cc::Build::new();

    for inc in INCLUDE_DIRS {
        build.include(inc);
    }

    for file in BUILD_FILES {
        add_file(&mut build, file);
    }

    if is_x64 {
        for file in BUILD_FILES_X64 {
            add_file(&mut build, file);
        }
    }

    build
        .file(asm_path)
        .define("WIN32", None)
        .define("_LIB", None)
        .define("FFI_BUILDING", None)
        .warnings(false)
        .compile("libffi");
}

pub fn probe_and_link() {
    // At the time of writing it wasn't clear if MSVC builds will support
    // dynamic linking of libffi; assuming it's even installed. To ensure
    // existing MSVC setups continue to work, we just compile libffi from source
    // and statically link it.
    build_and_link();
}

pub fn pre_process_asm(include_dirs: &[&str], target: &str, is_x64: bool) -> String {
    let file_name = if is_x64 { "win64_intel" } else { "sysv_intel" };

    let mut cmd = cc::windows_registry::find(&target, "cl.exe").expect("Could not locate cl.exe");
    cmd.env("INCLUDE", include_dirs.join(";"));

    cmd.arg("/EP");
    cmd.arg(format!("libffi/src/x86/{}.S", file_name));

    let out_path = format!("libffi/src/x86/{}.asm", file_name);
    let asm_file = fs::File::create(&out_path).expect("Could not create output file");

    cmd.stdout(asm_file);

    run_command("Pre-process ASM", &mut cmd);

    out_path
}
