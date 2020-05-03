#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]

// Based on https://github.com/serde-rs/serde/blob/v1.0.106/test_suite/build.rs

use std::{
    env,
    process::{Command, ExitStatus, Stdio},
};

#[cfg(not(windows))]
const CARGO_EXPAND: &str = "cargo-expand";

#[cfg(windows)]
const CARGO_EXPAND: &str = "cargo-expand.exe";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if Command::new(CARGO_EXPAND)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .as_ref()
        .map(ExitStatus::success)
        .unwrap_or(false)
    {
        println!("cargo:rustc-cfg=cargo_expand");
    }

    if env::var_os("CI").map_or(false, |v| v == "true") {
        println!("cargo:rustc-cfg=ci");
    }

    if is_nightly() {
        println!("cargo:rustc-cfg=nightly");
    }
}

fn is_nightly() -> bool {
    env::var_os("RUSTC")
        .and_then(|rustc| Command::new(rustc).arg("--version").output().ok())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or(false, |version| version.contains("nightly") || version.contains("dev"))
}
