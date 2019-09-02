// compile-fail

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

#[pin_project]
pub struct A {
    #[pin]
    field: u8,
}

#[pinned_drop]
fn do_drop(_x: Pin<&mut A>) -> bool {} //~ ERROR #[pinned_drop] function must return the unit type

#[pin_project]
pub struct B {
    #[pin]
    field: u8,
}

#[pinned_drop]
fn do_drop(_x: Pin<&mut B>) -> ((),) {} //~ ERROR #[pinned_drop] function must return the unit type

#[pin_project]
pub struct C {
    #[pin]
    field: u8,
}

#[pinned_drop]
fn do_drop(_x: Pin<&mut C>) -> () {} // OK

fn main() {}
