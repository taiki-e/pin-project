// NB: If you change this test, change 'trivial_bounds.rs' at the same time.

use std::marker::{PhantomData, PhantomPinned};

fn phantom_pinned() {
    struct Foo(PhantomPinned);

    impl Unpin for Foo where PhantomPinned: Unpin {} //~ ERROR E0277

    struct Wrapper<T>(T);

    impl<T> Unpin for Wrapper<T> where T: Unpin {}

    struct Bar(PhantomPinned);

    impl Unpin for Bar where Wrapper<PhantomPinned>: Unpin {} //~ ERROR E0277

    struct WrapperWithLifetime<'a, T>(PhantomData<&'a ()>, T);

    impl<T> Unpin for WrapperWithLifetime<'_, T> where T: Unpin {}

    struct Baz(PhantomPinned);

    impl<'a> Unpin for Baz where WrapperWithLifetime<'a, PhantomPinned>: Unpin {}
    // Ok
}

fn inner() {
    struct Inner(PhantomPinned);

    struct Foo(Inner);

    impl Unpin for Foo where Inner: Unpin {} //~ ERROR E0277

    struct Wrapper<T>(T);

    impl<T> Unpin for Wrapper<T> where T: Unpin {}

    struct Bar(Inner);

    impl Unpin for Bar where Wrapper<Inner>: Unpin {} //~ ERROR E0277

    struct WrapperWithLifetime<'a, T>(PhantomData<&'a ()>, T);

    impl<T> Unpin for WrapperWithLifetime<'_, T> where T: Unpin {}

    struct Baz(Inner);

    impl<'a> Unpin for Baz where WrapperWithLifetime<'a, Inner>: Unpin {} // Ok
}

fn main() {}
