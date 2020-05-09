#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

// default #[pin_project], PinnedDrop, Replace, and !Unpin are completely safe.

use pin_project::{pin_project, pinned_drop};
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

// UnsafeUnpin without UnsafeUnpin impl is also safe
#[pin_project(UnsafeUnpin)]
pub struct StructUnsafeUnpin<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

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

// UnsafeUnpin without UnsafeUnpin impl is also safe
#[pin_project(UnsafeUnpin)]
pub enum EnumUnsafeUnpin<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
}

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
