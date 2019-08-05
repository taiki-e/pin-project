#![recursion_limit = "128"]
#![no_std]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms)]
#![allow(dead_code)]

use pin_project::pin_projectable;

#[test]
fn unsafe_unpin() {
    #[pin_projectable(unsafe_Unpin)]
    pub struct Blah<T> {
        field_1: u8,
        #[pin]
        field_2: Option<T>,
    }
}
