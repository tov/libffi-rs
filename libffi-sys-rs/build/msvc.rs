use crate::common::*;

const INCLUDE_DIRS: &[&str] = &["libffi", "libffi/include"];

// headers for both 32-bit and 64-bit x86 are placed in libffi/src/x86
const INCLUDE_DIRS_X86: &[&str] = &["libffi/src/x86", "include/msvc/x86"];
const INCLUDE_DIRS_X86_64: &[&str] = &["libffi/src/x86", "include/msvc/x86_64"];
const INCLUDE_DIRS_AARCH64: &[&str] = &["libffi/src/aarch64", "include/msvc/aarch64"];

const BUILD_FILES: &[&str] = &[
    "libffi/src/tramp.c",
    "libffi/src/closures.c",
    "libffi/src/prep_cif.c",
    "libffi/src/raw_api.c",
    "libffi/src/types.c",
];
const BUILD_FILES_X86: &[&str] = &["libffi/src/x86/ffi.c"];
const BUILD_FILES_X86_64: &[&str] = &["libffi/src/x86/ffiw64.c"];
const BUILD_FILES_AARCH64: &[&str] = &["libffi/src/aarch64/ffi.c"];

fn unsupported(arch: &str) -> ! {
    panic!("Unsupported architecture: {arch}")
}

pub fn build_and_link() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    // we should collect all include dirs together with platform specific ones
    // to pass them over to the asm pre-processing step
    let mut all_includes: Vec<&str> = Vec::from_iter(INCLUDE_DIRS.iter().copied());

    all_includes.extend(match target_arch.as_str() {
        "x86" => INCLUDE_DIRS_X86,
        "x86_64" => INCLUDE_DIRS_X86_64,
        "aarch64" => INCLUDE_DIRS_AARCH64,
        _ => unsupported(&target_arch),
    });

    let asm_path = pre_process_asm(all_includes.as_slice(), &target_arch);

    cc::Build::new()
        .includes(&all_includes)
        .files(BUILD_FILES)
        .files(match target_arch.as_str() {
            "x86" => BUILD_FILES_X86,
            "x86_64" => BUILD_FILES_X86_64,
            "aarch64" => BUILD_FILES_AARCH64,
            _ => unsupported(&target_arch),
        })
        .file(asm_path)
        .define("WIN32", None)
        .define("_LIB", None)
        .define("FFI_BUILDING", None)
        .define("FFI_STATIC_BUILD", None)
        .pic(true)
        .warnings(false)
        .compile("libffi");

    println!("cargo::rerun-if-changed=build/");
    println!("cargo::rerun-if-changed=libffi/include");
    println!("cargo::rerun-if-changed=libffi/src");
}

pub fn pre_process_asm(include_dirs: &[&str], target_arch: &str) -> String {
    let folder_name = match target_arch {
        "x86" | "x86_64" => "x86",
        "aarch64" => "aarch64",
        _ => unsupported(target_arch),
    };

    let file_name = match target_arch {
        "x86" => "sysv_intel",
        "x86_64" => "win64_intel",
        "aarch64" => "win64_armasm",
        _ => unsupported(target_arch),
    };

    let in_file = format!("libffi/src/{folder_name}/{file_name}.S");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = format!("{out_dir}/processed_asm.asm");
    let out_file = fs::File::create(&out_path).unwrap();

    let mut cmd = cc::Build::new()
        .includes(include_dirs)
        .get_compiler()
        .to_command();

    cmd.arg("/EP");
    cmd.arg(in_file);

    cmd.stdout(out_file);

    run_command("Pre-process ASM", &mut cmd);

    out_path
}
