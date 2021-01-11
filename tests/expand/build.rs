#![warn(rust_2018_idioms, single_use_lifetimes)]

use std::{
    env,
    process::{Command, ExitStatus, Stdio},
};

fn main() {
    if !is_nightly() {
        return;
    }

    let cargo = &*env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    if !has_command(&[cargo, "expand"]) || !has_command(&[cargo, "fmt"]) {
        println!("cargo:warning=rustfmt or cargo-expand not found, skipping expandtest");
    }
}

fn is_nightly() -> bool {
    env::var_os("RUSTC")
        .and_then(|rustc| Command::new(rustc).arg("--version").output().ok())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or(false, |version| version.contains("nightly") || version.contains("dev"))
}

fn has_command(command: &[&str]) -> bool {
    Command::new(command[0])
        .args(&command[1..])
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .as_ref()
        .map(ExitStatus::success)
        .unwrap_or(false)
}
