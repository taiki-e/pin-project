// Based on https://stackoverflow.com/a/49250753/1290530

use std::env;

use rustc_version::{version_meta, Channel};

fn main() {
    // Set cfg flags depending on release channel
    match version_meta().unwrap().channel {
        // Enable our feature on nightly, or when using a
        // locally build rustc, unless `-Zallow-features`
        // in RUSTFLAGS disallows unstable features.
        Channel::Nightly | Channel::Dev if feature_allowed("proc_macro_def_site") => {
            println!("cargo:rustc-cfg=proc_macro_def_site");
        }
        _ => {}
    }
}

// Based on https://github.com/alexcrichton/proc-macro2/pull/176
fn feature_allowed(feature: &str) -> bool {
    if let Some(rustflags) = env::var_os("RUSTFLAGS") {
        for mut flag in rustflags.to_string_lossy().split(' ') {
            if flag.starts_with("-Z") {
                flag = &flag["-Z".len()..];
            }
            if flag.starts_with("allow-features=") {
                flag = &flag["allow-features=".len()..];
                return flag.split(',').any(|allowed| allowed == feature);
            }
        }
    }

    // No allow-features= flag, allowed by default.
    true
}
