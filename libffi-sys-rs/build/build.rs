use std::env;

mod common;

mod msvc;
mod not_msvc;

fn main() {
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();

    if target_env == "msvc" {
        msvc::build_and_link();
    } else if env::var_os("CARGO_FEATURE_SYSTEM").is_some() {
        not_msvc::probe_and_link();
    } else {
        not_msvc::build_and_link();
    }
}
