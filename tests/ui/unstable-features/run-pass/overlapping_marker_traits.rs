// NB: If you change this test, change 'overlapping_marker_traits-feature-gate.rs' at the same time.

// overlapping_marker_traits
// Tracking issue: https://github.com/rust-lang/rust/issues/29864
#![feature(overlapping_marker_traits)]

// See https://github.com/rust-lang/rust/issues/29864#issuecomment-515780867.

use pin_project::pin_project;
use std::marker::PhantomPinned;

#[pin_project]
struct Foo<T> {
    #[pin]
    x: T,
}

// unsound Unpin impl
impl<T> Unpin for Foo<T> {}

fn is_unpin<T: Unpin>() {}

fn main() {
    is_unpin::<Foo<PhantomPinned>>()
}
