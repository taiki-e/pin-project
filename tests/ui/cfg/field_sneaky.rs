#![feature(optin_builtin_traits)]
#![feature(trivial_bounds)]

#[macro_use]
extern crate auxiliary_macros;

use pin_project::pin_project;

fn is_unpin<T: Unpin>() {}

#[pin_project]
#[add_pinned_field]
struct Foo {
    #[pin]
    field: u32,
}

fn foo() {
    is_unpin::<Foo>(); //~ ERROR E0277
}

fn main() {}
