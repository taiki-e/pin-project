#![warn(rust_2018_idioms, single_use_lifetimes)]

fn main() {
    if rustversion::cfg!(nightly) {
        println!("cargo:rustc-cfg=nightly");
    }
}
