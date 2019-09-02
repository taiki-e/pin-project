// compile-fail

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

#[pin_project]
pub struct Foo {
    #[pin]
    field: u8,
}

#[pinned_drop(foo)] //~ ERROR unexpected token
fn do_drop(_x: Pin<&mut Foo>) {}

#[pin_project]
pub struct Bar {
    #[pin]
    field: u8,
}

#[pinned_drop]
fn do_drop(_x: &mut Bar) {} //~ ERROR #[pinned_drop] function must take a argument `Pin<&mut Type>`

fn main() {}
