use pin_project::{pin_project, UnsafeUnpin};

#[pin_project(UnsafeUnpin)]
pub struct Foo<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

unsafe impl<T: Unpin, U> UnsafeUnpin for Foo<T, U> {}

fn main() {}
