#![warn(rust_2018_idioms, single_use_lifetimes)]

#[cfg(not(ci))]
use macrotest::expand;
#[cfg(ci)]
use macrotest::expand_without_refresh as expand;
use std::env;

#[cfg_attr(not(expandtest), ignore)]
#[test]
fn expandtest() {
    if !cfg!(ci) {
        env::set_var("MACROTEST", "overwrite");
    }

    expand("tests/expand/*.rs");
}
