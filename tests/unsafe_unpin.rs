#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use pin_project::{pin_project, UnsafeUnpin};

#[test]
fn unsafe_unpin() {
    #[pin_project(UnsafeUnpin)]
    pub struct Blah<T> {
        field1: u8,
        #[pin]
        field2: Option<T>,
    }

    #[allow(unsafe_code)]
    unsafe impl<T: Unpin> UnsafeUnpin for Blah<T> {}
}
