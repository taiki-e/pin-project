#![no_std]

extern crate pin_project;

use core::pin::Pin;
use pin_project::{pin_project, pinned_drop, UnsafeUnpin};

#[pin_project]
pub struct StructDefault<T, U> {
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

#[pin_project]
pub enum EnumDefault<T, U> {
    Variant {
        #[pin]
        pinned: T,
        unpinned: U,
    },
}

#[pin_project(UnsafeUnpin)]
pub enum EnumUnsafeUnpin<T, U> {
    Variant {
        #[pin]
        pinned: T,
        unpinned: U,
    },
}

unsafe impl<T: Unpin, U> UnsafeUnpin for EnumUnsafeUnpin<T, U> {}

#[pin_project(PinnedDrop)]
pub enum EnumPinnedDrop<T, U> {
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
pub enum EnumReplace<T, U> {
    Variant {
        #[pin]
        pinned: T,
        unpinned: U,
    },
}
