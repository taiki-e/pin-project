// run-pass
// aux-build:sneaky_macro.rs

#[macro_use]
extern crate sneaky_macro;

use pin_project::pin_project;

// #[cfg_attr(any(), repr(packed))]
#[pin_project]
#[hidden_repr_cfg_any(packed)]
struct Foo {
    #[pin]
    field: u32,
}

fn main() {}
