pub use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[track_caller]
pub fn run_command(which: &'static str, cmd: &mut Command) {
    match cmd.status() {
        Ok(status) if status.success() => return,
        Ok(status) => {
            println!("cargo:warning={} failed with {}", which, status);
            panic!("{}: {} ({:?})", which, status, cmd);
        }
        Err(err) => {
            println!("cargo:warning={} failed with error {}", which, err);
            panic!("{}: {} ({:?})", which, err, cmd);
        },
    }
}
