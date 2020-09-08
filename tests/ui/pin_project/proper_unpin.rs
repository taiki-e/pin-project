use pin_project::pin_project;
use std::marker::PhantomPinned;

struct Inner<T> {
    f: T,
}

#[pin_project]
struct Foo<T, U> {
    #[pin]
    f1: Inner<T>,
    f2: U,
}

#[pin_project]
struct TrivialBounds {
    #[pin]
    f: PhantomPinned,
}

#[pin_project]
struct Bar<'a, T, U> {
    #[pin]
    f1: &'a mut Inner<T>,
    f2: U,
}

fn is_unpin<T: Unpin>() {}

fn main() {
    is_unpin::<Foo<PhantomPinned, ()>>(); //~ ERROR E0277
    is_unpin::<Foo<(), PhantomPinned>>(); // Ok
    is_unpin::<Foo<PhantomPinned, PhantomPinned>>(); //~ ERROR E0277

    is_unpin::<TrivialBounds>(); //~ ERROR E0277

    is_unpin::<Bar<'_, PhantomPinned, PhantomPinned>>(); // Ok
}
