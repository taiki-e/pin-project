// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(not(miri))]
#![cfg(not(careful))]
#![warn(rust_2018_idioms, single_use_lifetimes)]

use std::env;

const PATH: &str = "tests/expand/**/*.rs";

#[rustversion::attr(not(nightly), ignore)]
#[test]
fn expandtest() {
    let args = &["--all-features"];
    if env::var_os("CI").is_some() {
        macrotest::expand_without_refresh_args(PATH, args);
    } else {
        env::set_var("MACROTEST", "overwrite");
        macrotest::expand_args(PATH, args);
    }
}
