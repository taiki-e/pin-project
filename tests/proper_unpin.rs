#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

#[macro_use]
mod auxiliary;

pub mod default {
    use pin_project::pin_project;
    use std::marker::PhantomPinned;

    struct Inner<T> {
        f: T,
    }

    assert_unpin!(Inner<()>);
    assert_not_unpin!(Inner<PhantomPinned>);

    #[pin_project]
    struct Foo<T, U> {
        #[pin]
        f1: Inner<T>,
        f2: U,
    }

    assert_unpin!(Foo<(), ()>);
    assert_unpin!(Foo<(), PhantomPinned>);
    assert_not_unpin!(Foo<PhantomPinned, ()>);
    assert_not_unpin!(Foo<PhantomPinned, PhantomPinned>);

    #[pin_project]
    struct TrivialBounds {
        #[pin]
        f: PhantomPinned,
    }

    assert_not_unpin!(TrivialBounds);

    #[pin_project]
    struct Bar<'a, T, U> {
        #[pin]
        f1: &'a mut Inner<T>,
        f2: U,
    }

    assert_unpin!(Bar<'_, PhantomPinned, PhantomPinned>);
}

pub mod cfg {
    use pin_project::pin_project;
    use std::marker::PhantomPinned;

    #[pin_project]
    struct Foo<T> {
        #[cfg(any())]
        #[pin]
        f: T,
        #[cfg(not(any()))]
        f: T,
    }

    assert_unpin!(Foo<PhantomPinned>);

    #[pin_project]
    struct Bar<T> {
        #[cfg(any())]
        f: T,
        #[cfg(not(any()))]
        #[pin]
        f: T,
    }

    assert_unpin!(Bar<()>);
    assert_not_unpin!(Bar<PhantomPinned>);
}

pub mod cfg_attr {
    use pin_project::pin_project;
    use std::marker::PhantomPinned;

    #[cfg_attr(any(), pin_project)]
    struct Foo<T> {
        f: T,
    }

    assert_unpin!(Foo<()>);
    assert_not_unpin!(Foo<PhantomPinned>);

    #[cfg_attr(not(any()), pin_project)]
    struct Bar<T> {
        #[cfg_attr(not(any()), pin)]
        f: T,
    }

    assert_unpin!(Bar<()>);
    assert_not_unpin!(Bar<PhantomPinned>);
}

// pin_project(!Unpin)
pub mod not_unpin {
    use pin_project::pin_project;
    use std::marker::PhantomPinned;

    struct Inner<T> {
        f: T,
    }

    #[pin_project(!Unpin)]
    struct Foo<T, U> {
        #[pin]
        inner: Inner<T>,
        other: U,
    }

    assert_not_unpin!(Foo<(), ()>);
    assert_not_unpin!(Foo<(), PhantomPinned>);
    assert_not_unpin!(Foo<PhantomPinned, ()>);
    assert_not_unpin!(Foo<PhantomPinned, PhantomPinned>);

    #[pin_project(!Unpin)]
    struct TrivialBounds {
        #[pin]
        f: PhantomPinned,
    }

    assert_not_unpin!(TrivialBounds);

    #[pin_project(!Unpin)]
    struct Bar<'a, T, U> {
        #[pin]
        inner: &'a mut Inner<T>,
        other: U,
    }

    assert_not_unpin!(Bar<'_, (), ()>);
}
