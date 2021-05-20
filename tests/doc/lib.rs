#![cfg(nightly)]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_variables)
    )
))]

// As `doc = include_str!` and `doc-comment` do not work with `cfg(test)`,
// and `cfg(doctest)` requires 1.40, these tests are split into this crate until
// MSRV increases.

// https://github.com/rust-lang/rust/issues/82768
#[cfg_attr(doctest, cfg_attr(doctest, doc = include_str!("../../README.md")))]
const _README: () = ();
