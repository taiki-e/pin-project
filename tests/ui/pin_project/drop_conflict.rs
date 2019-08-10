// compile-fail

#![deny(warnings, unsafe_code)]

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

#[pin_project] //~ ERROR E0119
struct Foo<T, U> {
    #[pin]
    future: T,
    field: U,
}

impl<T, U> Drop for Foo<T, U> {
    fn drop(&mut self) {}
}

#[pin_project(PinnedDrop)] //~ ERROR E0119
struct Bar<T, U> {
    #[pin]
    future: T,
    field: U,
}

#[pinned_drop]
fn do_drop<T, U>(this: Pin<&mut Bar<T, U>>) {}

impl<T, U> Drop for Bar<T, U> {
    fn drop(&mut self) {}
}

fn main() {}
