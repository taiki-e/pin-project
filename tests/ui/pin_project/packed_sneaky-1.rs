// compile-fail
// aux-build:sneaky_macro.rs

#[macro_use]
extern crate sneaky_macro;

use pin_project::{pin_project, pinned_drop, UnsafeUnpin};
use std::pin::Pin;

#[pin_project] //~ ERROR borrow of packed field is unsafe and requires unsafe function or block
#[hidden_repr(packed)]
struct A {
    #[pin]
    field: u32,
}

#[pin_project(UnsafeUnpin)] //~ ERROR borrow of packed field is unsafe and requires unsafe function or block
#[hidden_repr(packed)]
struct C {
    #[pin]
    field: u32,
}

unsafe impl UnsafeUnpin for C {}

#[pin_project(PinnedDrop)] //~ ERROR borrow of packed field is unsafe and requires unsafe function or block
#[hidden_repr(packed)]
struct D {
    #[pin]
    field: u32,
}

#[pinned_drop]
impl PinnedDrop for D {
    fn drop(self: Pin<&mut Self>) {}
}

fn main() {}
