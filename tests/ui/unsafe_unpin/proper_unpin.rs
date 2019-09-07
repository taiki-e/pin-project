// compile-fail

use pin_project::{pin_project, UnsafeUnpin};
use std::{marker::PhantomPinned, pin::Pin};

#[pin_project(UnsafeUnpin)]
struct Foo<T, U> {
    #[pin]
    inner: T,
    other: U,
}

unsafe impl<T: Unpin, U> UnsafeUnpin for Foo<T, U> {}

fn is_unpin<T: Unpin>() {}

fn bar<T, U>() {
    is_unpin::<Foo<PhantomPinned, U>>(); //~ ERROR E0277
}

fn main() {}
