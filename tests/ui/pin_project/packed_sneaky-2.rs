// compile-fail
// aux-build:sneaky_macro.rs

#[macro_use]
extern crate sneaky_macro;

use pin_project::pin_project;

hidden_repr_macro! { //~ ERROR borrow of packed field is unsafe and requires unsafe function or block
    #[pin_project]
    struct B {
        #[pin]
        field: u32,
    }
}

fn main() {}
