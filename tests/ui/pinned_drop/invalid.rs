// compile-fail

#![deny(warnings, unsafe_code)]

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

#[pin_project]
pub struct Foo<'a> {
    was_dropped: &'a mut bool,
    #[pin]
    field_2: u8,
}

#[pinned_drop(foo)] //~ ERROR unexpected token
fn do_drop(_foo: Pin<&mut Foo<'_>>) {}

#[pin_project]
pub struct Bar<'a> {
    was_dropped: &'a mut bool,
    #[pin]
    field_2: u8,
}

#[pinned_drop]
fn do_drop(_bar: &mut Bar<'_>) {} //~ ERROR #[pinned_drop] function must take a argument `Pin<&mut Type>`

fn main() {}
