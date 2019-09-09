// See ./pinned_drop-expanded.rs for generated code.

#![allow(dead_code, unused_imports)]

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

#[pin_project(PinnedDrop)]
pub struct Foo<'a, T> {
    was_dropped: &'a mut bool,
    #[pin]
    field: T,
}

#[pinned_drop]
fn drop_foo<T>(mut this: Pin<&mut Foo<'_, T>>) {
    **this.project().was_dropped = true;
}

fn main() {}
