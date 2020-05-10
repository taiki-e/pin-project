#![cfg(nightly)]
#![warn(rust_2018_idioms, single_use_lifetimes)]

#[cfg_attr(any(not(cargo_expand), all(ci, not(target_os = "linux"))), ignore)]
#[test]
fn expandtest() {
    #[cfg(target_os = "linux")] // FIXME
    #[cfg(ci)]
    macrotest::expand_without_refresh("tests/expand/*.rs");
    #[cfg(not(ci))]
    macrotest::expand("tests/expand/*.rs");
}
