// run-pass

#![deny(warnings, unsafe_code)]

use pin_project::pin_project;

// FIXME
#[test]
fn unsafe_unpin() {
    #[pin_project(unsafe_Unpin)]
    pub struct Blah<T> {
        field_1: u8,
        #[pin]
        field_2: Option<T>,
    }
}

fn main() {}
