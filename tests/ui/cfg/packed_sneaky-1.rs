// compile-fail
// aux-build:sneaky_macro.rs

#[macro_use]
extern crate sneaky_macro;

use pin_project::pin_project;

#[pin_project] // Pass
#[hidden_repr(packed)]
struct Foo {
    #[cfg(any())]
    #[pin]
    field: u32,
    #[cfg(not(any()))]
    #[pin]
    field: u8,
}

#[pin_project] //~ ERROR borrow of packed field is unsafe and requires unsafe function or block
#[hidden_repr(packed)]
struct Bar {
    #[cfg(not(any()))]
    #[pin]
    field: u32,
    #[cfg(any())]
    #[pin]
    field: u8,
}

fn main() {}
