// NB: If you change this test, change 'trivial_bounds-feature-gate.rs' at the same time.

// trivial_bounds
// Tracking issue: https://github.com/rust-lang/rust/issues/48214
#![feature(trivial_bounds)]
#![deny(trivial_bounds)]

use std::marker::{PhantomData, PhantomPinned};

fn inner() {
    struct Inner(PhantomPinned);

    struct Foo(Inner);

    impl Unpin for Foo where Inner: Unpin {} //~ ERROR std::marker::Unpin does not depend on any type or lifetime parameters

    struct Wrapper<T>(T);

    impl<T> Unpin for Wrapper<T> where T: Unpin {}

    struct Bar(Inner);

    impl Unpin for Bar where Wrapper<Inner>: Unpin {} //~ ERROR std::marker::Unpin does not depend on any type or lifetime parameters

    struct WrapperWithLifetime<'a, T>(PhantomData<&'a ()>, T);

    impl<T> Unpin for WrapperWithLifetime<'_, T> where T: Unpin {}

    struct Baz(Inner);

    impl<'a> Unpin for Baz where WrapperWithLifetime<'a, Inner>: Unpin {} // Ok
}

fn main() {}
