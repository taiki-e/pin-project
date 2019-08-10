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

#[pinned_drop]
fn do_drop(foo: Pin<&mut Foo<'_>>) -> bool {
    //~ ERROR #[pinned_drop] function must return the unit type
    **foo.project().was_dropped = true;
    true
}

#[pin_project]
pub struct Bar<'a> {
    was_dropped: &'a mut bool,
    #[pin]
    field_2: u8,
}

// OK
#[pinned_drop]
fn do_drop(foo: Pin<&mut Bar<'_>>) -> () {
    **foo.project().was_dropped = true;
}

fn main() {}
