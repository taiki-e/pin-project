#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

// default #[pin_project], PinnedDrop, Replace are completely safe.

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

#[test]
fn test() {
    #[pin_project]
    struct Struct1<T, U> {
        #[pin]
        pinned: T,
        unpinned: U,
    }

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
