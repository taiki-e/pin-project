// compile-fail
// aux-build:sneaky_macro.rs

#[macro_use]
extern crate sneaky_macro;

use pin_project::pin_project;

#[pin_project] //~ ERROR borrow of packed field is unsafe and requires unsafe function or block
#[hidden_repr(packed)]
struct Foo {
    #[pin]
    field: u32,
}

fn main() {}
