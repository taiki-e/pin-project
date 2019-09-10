// compile-fail
// aux-build:sneaky_macro.rs

#[macro_use]
extern crate sneaky_macro;

use pin_project::pin_project;

// #[cfg_attr(not(any()), repr(packed))]
#[pin_project] //~ ERROR borrow of packed field is unsafe and requires unsafe function or block
#[hidden_repr_cfg_not_any(packed)]
struct Foo {
    #[pin]
    field: u32,
}

fn main() {}
