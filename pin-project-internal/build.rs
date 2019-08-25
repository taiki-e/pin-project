// Based on https://stackoverflow.com/a/49250753/1290530

use rustc_version::{version_meta, Channel};

fn main() {
    // Set cfg flags depending on release channel
    match version_meta().unwrap().channel {
        // Enable our feature on nightly, or when using a
        // locally build rustc
        Channel::Nightly | Channel::Dev => {
            println!("cargo:rustc-cfg=feature=\"RUSTC_IS_NIGHTLY\"");
        }
        _ => {}
    }
}
