#![warn(rust_2018_idioms, single_use_lifetimes)]

#[cfg(not(ci))]
use macrotest::expand;
#[cfg(ci)]
use macrotest::expand_without_refresh as expand;

#[cfg_attr(not(expandtest), ignore)]
#[test]
fn expandtest() {
    expand("tests/expand/*.rs");
}
