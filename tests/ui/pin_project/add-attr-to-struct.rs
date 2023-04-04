use std::marker::PhantomPinned;

use auxiliary_macro::add_pin_attr;
use pin_project::pin_project;

#[pin_project]
struct Foo {
    #[pin]
    #[pin] //~ ERROR duplicate #[pin] attribute
    f: PhantomPinned,
}

#[add_pin_attr(struct)] //~ ERROR #[pin] attribute may only be used on fields of structs or variants
#[pin_project]
struct Bar {
    #[pin]
    f: PhantomPinned,
}

fn main() {}
