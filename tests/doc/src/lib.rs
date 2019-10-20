#![cfg(nightly)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms, single_use_lifetimes), allow(dead_code))
))]
#![feature(external_doc)]

// As `external_doc` and `doc-comment` do not work with `cfg(test)`,
// these tests are split into this crate.
// Refs:
// * https://github.com/rust-lang/rust/issues/62210
// * https://github.com/rust-lang/rust/pull/63803

#[doc(include = "../../../README.md")]
const _README: () = ();
