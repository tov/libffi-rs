pub use std::{env, fs, process::Command};

#[track_caller]
pub fn run_command(which: &'static str, cmd: &mut Command) {
    match cmd.status() {
        Ok(status) if status.success() => (),
        Ok(status) => {
            println!("cargo:warning={which} failed with {status}");
            panic!("{which}: {status} ({cmd:?})");
        }
        Err(err) => {
            println!("cargo:warning={which} failed with error {err}");
            panic!("{which}: {err} ({cmd:?})");
        }
    }
}
