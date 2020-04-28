#![cfg(nightly)]
#![cfg(target_os = "linux")]

#[cfg_attr(not(cargo_expand), ignore)]
#[test]
fn expandtest() {
    macrotest::expand("tests/enum/*.rs");
    macrotest::expand("tests/struct/*.rs");
}
