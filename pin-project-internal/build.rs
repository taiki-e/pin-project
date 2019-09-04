use std::env;

fn main() {
    // While this crate supports stable Rust, it currently requires
    // nightly Rust in order for rustdoc to correctly document auto-generated
    // `Unpin` impls. This does not affect the runtime functionality of this crate,
    // nor does it affect the safety of the api provided by this crate.
    //
    // This is disabled by default and can be enabled using
    // `--cfg pin_project_show_unpin_struct` in RUSTFLAGS.
    //
    // Refs:
    // * https://github.com/taiki-e/pin-project/pull/53#issuecomment-525906867
    // * https://github.com/taiki-e/pin-project/pull/70
    // * https://github.com/rust-lang/rust/issues/63281
    if cfg!(pin_project_show_unpin_struct) && feature_allowed("proc_macro_def_site") {
        println!("cargo:rustc-cfg=proc_macro_def_site");
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
