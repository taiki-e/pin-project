#![no_std]

use core::pin::Pin;
use pin_project::{pin_project, pinned_drop, UnsafeUnpin};

#[pin_project]
pub struct Struct1<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[pin_project(UnsafeUnpin)]
pub struct Struct2<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

unsafe impl<T: Unpin, U> UnsafeUnpin for Struct2<T, U> {}

#[pin_project(PinnedDrop)]
pub struct Struct3<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[pinned_drop]
impl<T, U> PinnedDrop for Struct3<T, U> {
    fn drop(self: Pin<&mut Self>) {}
}

#[pin_project(Replace)]
pub struct Struct4<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}
