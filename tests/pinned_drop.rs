#![no_std]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use core::pin::Pin;
use pin_project::{pin_project, pinned_drop};

#[test]
fn safe_project() {
    #[pin_project(PinnedDrop)]
    pub struct Foo<'a> {
        was_dropped: &'a mut bool,
        #[pin]
        field: u8,
    }

    #[pinned_drop]
    fn do_drop(mut foo: Pin<&mut Foo<'_>>) {
        **foo.project().was_dropped = true;
    }

    let mut was_dropped = false;
    drop(Foo { was_dropped: &mut was_dropped, field: 42 });
    assert!(was_dropped);
}

#[test]
fn overlapping_drop_fn_names() {
    #[pin_project(PinnedDrop)]
    pub struct Foo {
        #[pin]
        field: u8,
    }

    #[pinned_drop]
    fn do_drop(_: Pin<&mut Foo>) {}

    #[pin_project(PinnedDrop)]
    pub struct Bar {
        #[pin]
        field: u8,
    }

    #[pinned_drop]
    fn do_drop(_: Pin<&mut Bar>) {}
}
