#![warn(rust_2018_idioms, single_use_lifetimes)]

#[cfg_attr(not(expandtest), ignore)]
#[test]
fn expandtest() {
    #[cfg(ci)]
    macrotest::expand_without_refresh("tests/expand/*.rs");
    #[cfg(not(ci))]
    macrotest::expand("tests/expand/*.rs");
}
