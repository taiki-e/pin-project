#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

// default #[pin_project], PinnedDrop, Replace are completely safe.

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

#[pin_project]
struct StructDefault<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

// UnsafeUnpin without UnsafeUnpin impl is also safe
#[pin_project(UnsafeUnpin)]
struct StructUnsafeUnpin<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

#[pin_project(PinnedDrop)]
struct StructPinnedDrop<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

#[pinned_drop]
impl<T, U> PinnedDrop for StructPinnedDrop<T, U> {
    fn drop(self: Pin<&mut Self>) {}
}

#[pin_project(Replace)]
struct StructReplace<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

#[pin_project]
enum EnumDefault<T, U> {
    Variant {
        #[pin]
        pinned: T,
        unpinned: U,
    },
}

// UnsafeUnpin without UnsafeUnpin impl is also safe
#[pin_project(UnsafeUnpin)]
enum EnumUnsafeUnpin<T, U> {
    Variant {
        #[pin]
        pinned: T,
        unpinned: U,
    },
}

#[pin_project(PinnedDrop)]
enum EnumPinnedDrop<T, U> {
    Variant {
        #[pin]
        pinned: T,
        unpinned: U,
    },
}

#[pinned_drop]
impl<T, U> PinnedDrop for EnumPinnedDrop<T, U> {
    fn drop(self: Pin<&mut Self>) {}
}

#[pin_project(Replace)]
enum EnumReplace<T, U> {
    Variant {
        #[pin]
        pinned: T,
        unpinned: U,
    },
}

#[test]
fn test() {}
