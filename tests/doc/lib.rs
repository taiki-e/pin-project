#![cfg(nightly)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms, single_use_lifetimes), allow(dead_code))
))]
#![cfg_attr(doctest, feature(external_doc))]

// As `feature(external_doc)` and `doc-comment` do not work with `cfg(test)`,
// and `cfg(doctest)` requires 1.40, these tests are split into this crate until
// MSRV increases.

#[cfg(doctest)]
#[doc(include = "../../README.md")]
const _README: () = ();
