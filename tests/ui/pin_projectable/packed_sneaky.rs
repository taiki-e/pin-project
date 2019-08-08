// compile-fail
// aux-build:sneaky_macro.rs

#[macro_use]
extern crate sneaky_macro;

use pin_project::pin_projectable;

#[pin_projectable] //~ ERROR borrow of packed field is unsafe and requires unsafe function or block
#[hidden_repr(packed)]
struct Bar {
    #[pin]
    field: u32
}

fn main() {}
