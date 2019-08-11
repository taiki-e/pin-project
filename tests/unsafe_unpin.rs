#![no_std]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms)]
#![allow(dead_code)]

use pin_project::{pin_project, UnsafeUnpin};

#[test]
fn unsafe_unpin() {
    #[pin_project(unsafe_Unpin)]
    pub struct Blah<T> {
        field_1: u8,
        #[pin]
        field_2: Option<T>,
    }

    #[allow(unsafe_code)]
    unsafe impl<T: Unpin> UnsafeUnpin for Blah<T> {}
}
