use pin_project::{pin_project, UnsafeUnpin};
use std::marker::PhantomPinned;

fn is_unpin<T: Unpin>() {}

#[pin_project(UnsafeUnpin)]
struct Blah<T, U> {
    f1: U,
    #[pin]
    f2: T,
}

unsafe impl<T: Unpin, U> UnsafeUnpin for Blah<T, U> {}

#[pin_project(UnsafeUnpin)]
struct TrivialBounds {
    #[pin]
    f: PhantomPinned,
}

#[pin_project(UnsafeUnpin)]
struct OverlappingLifetimeNames<'pin, T, U> {
    #[pin]
    f1: U,
    #[pin]
    f2: Option<T>,
    f3: &'pin (),
}

unsafe impl<T: Unpin, U: Unpin> UnsafeUnpin for OverlappingLifetimeNames<'_, T, U> {}

fn main() {
    is_unpin::<Blah<PhantomPinned, ()>>(); //~ ERROR E0277
    is_unpin::<Blah<(), PhantomPinned>>(); // Ok
    is_unpin::<Blah<PhantomPinned, PhantomPinned>>(); //~ ERROR E0277

    is_unpin::<TrivialBounds>(); //~ ERROR E0277

    is_unpin::<OverlappingLifetimeNames<'_, PhantomPinned, ()>>(); //~ ERROR E0277
    is_unpin::<OverlappingLifetimeNames<'_, (), PhantomPinned>>(); //~ ERROR E0277
}
