#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]

use std::{env, process::Command, str};

// The rustc-cfg strings below are *not* public API. Please let us know by
// opening a GitHub issue if your build environment requires some way to enable
// these cfgs other than by executing our build script.
fn main() {
    let minor = match rustc_minor_version() {
        Some(minor) => minor,
        None => return,
    };

    // Underscore const names requires Rust 1.37:
    // https://github.com/rust-lang/rust/pull/61347
    if minor >= 37 {
        println!("cargo:rustc-cfg=underscore_consts");
    }
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).args(&["--version", "--verbose"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let output = str::from_utf8(&output.stdout).ok()?;

    // Find the release line in the verbose version output.
    let release = output
        .lines()
        .find(|line| line.starts_with("release: "))
        .map(|line| &line["release: ".len()..])?;

    // Split the version and channel info.
    let mut version_channel = release.split('-');
    let version = version_channel.next().unwrap();
    let _channel = version_channel.next();

    // Split the version into semver components.
    let mut digits = version.splitn(3, '.');
    let major = digits.next()?;
    if major != "1" {
        return None;
    }
    let minor = digits.next()?.parse().ok()?;
    let _patch = digits.next()?;
    Some(minor)
}
