#![warn(rust_2018_idioms, single_use_lifetimes)]

// See https://github.com/rust-lang/pin-utils/pull/26#discussion_r344491597
//
// Note: If the proc-macro does not depend on its own items, it may be preferable not to
//       support overwriting the name of core/std crate for compatibility with reexport.
#[allow(unused_extern_crates)]
extern crate pin_project as core;

// Dummy module to check that the expansion refers to the crate.
mod pin_project {}

use ::pin_project::{pin_project, pinned_drop, UnsafeUnpin};
use std::pin::Pin;

#[pin_project]
pub struct StructDefault<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[pin_project(PinnedDrop)]
pub struct StructPinnedDrop<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[pinned_drop]
impl<T, U> PinnedDrop for StructPinnedDrop<T, U> {
    fn drop(self: Pin<&mut Self>) {}
}

#[pin_project(Replace)]
pub struct StructReplace<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[pin_project(UnsafeUnpin)]
pub struct StructUnsafeUnpin<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

unsafe impl<T: Unpin, U> UnsafeUnpin for StructUnsafeUnpin<T, U> {}

#[pin_project(!Unpin)]
pub struct StructNotUnpin<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[pin_project]
pub enum EnumDefault<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
}

#[pin_project(PinnedDrop)]
pub enum EnumPinnedDrop<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
}

#[pinned_drop]
impl<T, U> PinnedDrop for EnumPinnedDrop<T, U> {
    fn drop(self: Pin<&mut Self>) {}
}

#[pin_project(Replace)]
pub enum EnumReplace<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
}

#[pin_project(UnsafeUnpin)]
pub enum EnumUnsafeUnpin<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
}

unsafe impl<T: Unpin, U> UnsafeUnpin for EnumUnsafeUnpin<T, U> {}

#[pin_project(!Unpin)]
pub enum EnumNotUnpin<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
}

#[test]
fn test() {}
