#![recursion_limit = "128"]
#![no_std]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms)]
#![allow(dead_code)]

use core::pin::Pin;
use pin_project::{pin_project, pinned_drop};

#[pin_project(PinnedDrop)]
pub struct Foo<'a> {
    was_dropped: &'a mut bool,
    #[pin]
    field_2: u8,
}

#[pinned_drop]
fn do_drop(foo: Pin<&mut Foo<'_>>) {
    **foo.project().was_dropped = true;
}

#[test]
fn safe_project() {
    let mut was_dropped = false;
    drop(Foo { was_dropped: &mut was_dropped, field_2: 42 });
    assert!(was_dropped);
}
