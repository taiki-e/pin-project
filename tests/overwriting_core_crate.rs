#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

// See https://github.com/rust-lang/pin-utils/pull/26#discussion_r344491597
//
// Note: If the proc-macro does not depend on its own items, it may be preferable not to
//       support overwriting the name of core/std crate for compatibility with reexport.
#[allow(unused_extern_crates)]
extern crate pin_project as core;

// Dummy module to check that the expansion refer to pin-project crate
mod pin_project {}

use ::pin_project::{pin_project, pinned_drop, UnsafeUnpin};
use std::pin::Pin;

#[pin_project]
struct StructDefault<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

#[pin_project(UnsafeUnpin)]
struct StructUnsafeUnpin<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

unsafe impl<T: Unpin, U> UnsafeUnpin for StructUnsafeUnpin<T, U> {}

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
struct StructMutMut<'a, T, U> {
    #[pin]
    pinned: &'a mut T,
    unpinned: &'a mut U,
}

#[pin_project]
enum EnumDefault<T, U> {
    Variant {
        #[pin]
        pinned: T,
        unpinned: U,
    },
}

#[pin_project(UnsafeUnpin)]
enum EnumUnsafeUnpin<T, U> {
    Variant {
        #[pin]
        pinned: T,
        unpinned: U,
    },
}

unsafe impl<T: Unpin, U> UnsafeUnpin for EnumUnsafeUnpin<T, U> {}

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

#[pin_project]
enum EnumMutMut<'a, T, U> {
    Variant {
        #[pin]
        pinned: &'a mut T,
        unpinned: &'a mut U,
    },
}

#[test]
fn test() {}
