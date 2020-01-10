pub use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub struct IncludePaths(pub Vec<PathBuf>);

pub fn run_command(which: &'static str, cmd: &mut Command) {
    assert!(cmd.status().expect(which).success(), which);
}

