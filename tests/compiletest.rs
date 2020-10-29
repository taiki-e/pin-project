#![cfg(not(miri))]
#![warn(rust_2018_idioms, single_use_lifetimes)]

#[rustversion::attr(before(2020-10-28), ignore)] // Note: This date is the day before the toolchain date.
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/cfg/*.rs");
    t.compile_fail("tests/ui/not_unpin/*.rs");
    t.compile_fail("tests/ui/pin_project/*.rs");
    t.compile_fail("tests/ui/pinned_drop/*.rs");
    t.compile_fail("tests/ui/unsafe_unpin/*.rs");
    t.compile_fail("tests/ui/unstable-features/*.rs");
}
