#[macro_use]
extern crate auxiliary_macros;

use pin_project::pin_project;

#[pin_project]
// Generate #[cfg_attr(not(any()), repr(packed))]
#[hidden_repr_cfg_not_any(packed)] //~ ERROR may not be used on #[repr(packed)] types
struct Foo {
    #[pin]
    field: u32,
}

fn main() {}
