#![cfg(nightly)]
#![warn(rust_2018_idioms, single_use_lifetimes)]

#[cfg_attr(any(not(cargo_expand), all(ci, not(target_os = "linux"))), ignore)]
#[test]
fn expandtest() {
    macrotest::expand("tests/enum/*.rs");
    macrotest::expand("tests/struct/*.rs");
}
