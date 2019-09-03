// Based on https://stackoverflow.com/a/49250753/1290530

use std::env;

use rustc_version::{version_meta, Channel};

fn main() {
    // Set cfg flags depending on release channel
    match version_meta().unwrap().channel {
        // Enable our feature on nightly, or when using a
        // locally build rustc.
        //
        // This is intended to avoid the issue that cannot know the actual
        // trait implementation bounds of the `Unpin` implementation from the
        // document of generated code.
        // See [taiki-e/pin-project#53] and [rust-lang/rust#63281] for more details.
        //
        // [taiki-e/pin-project#53]: https://github.com/taiki-e/pin-project/pull/53#issuecomment-525906867
        // [rust-lang/rust#63281]: https://github.com/rust-lang/rust/issues/63281
        //
        // You can opt-out of this in one of the followg ways:
        // * Use `--cfg pin_project_stable_docs` in RUSTFLAGS.
        //   ```toml
        //   # in Cargo.toml
        //   [package.metadata.docs.rs]
        //   rustdoc-args = ["--cfg", "pin_project_stable_docs"]
        //   ```
        // * Use `-Zallow-features` in RUSTFLAGS to disallow unstable features.
        Channel::Nightly | Channel::Dev
            if feature_allowed("proc_macro_def_site") && !cfg!(pin_project_stable_docs) =>
        {
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
