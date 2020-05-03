#![allow(dead_code)]

extern crate pin_project;

use pin_project::{pin_project, pinned_drop, UnsafeUnpin};
use std::pin::Pin;

#[test]
fn test() {
    #[pin_project]
    struct Struct1<T, U> {
        #[pin]
        pinned: T,
        unpinned: U,
    }

    #[pin_project(UnsafeUnpin)]
    struct Struct2<T, U> {
        #[pin]
        pinned: T,
        unpinned: U,
    }

    unsafe impl<T: Unpin, U> UnsafeUnpin for Struct2<T, U> {}

    #[pin_project(PinnedDrop)]
    struct Struct3<T, U> {
        #[pin]
        pinned: T,
        unpinned: U,
    }

    #[pinned_drop]
    impl<T, U> PinnedDrop for Struct3<T, U> {
        fn drop(self: Pin<&mut Self>) {}
    }

    #[pin_project(Replace)]
    struct Struct4<T, U> {
        #[pin]
        pinned: T,
        unpinned: U,
    }
}
