// compile-fail

use pin_project::pin_project;
use std::{marker::PhantomPinned, pin::Pin};

#[pin_project]
struct Foo<T> {
    #[cfg(any())]
    #[pin]
    inner: T,
    #[cfg(not(any()))]
    inner: T,
}

#[pin_project]
struct Bar<T> {
    #[cfg(any())]
    inner: T,
    #[cfg(not(any()))]
    #[pin]
    inner: T,
}

fn is_unpin<T: Unpin>() {}

fn baz<T, U>() {
    is_unpin::<Foo<PhantomPinned>>(); // Pass
    is_unpin::<Bar<PhantomPinned>>(); //~ ERROR E0277
}

fn main() {}
