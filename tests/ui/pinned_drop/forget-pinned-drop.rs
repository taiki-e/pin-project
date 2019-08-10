// compile-fail

#![deny(warnings, unsafe_code)]

use core::pin::Pin;
use pin_project::{pin_project, pinned_drop};

#[pin_project(PinnedDrop)]
pub struct Foo<'a> {
    was_dropped: &'a mut bool,
    #[pin]
    field_2: u8,
}

fn main() {}
