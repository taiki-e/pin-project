#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use pin_project::{pin_project, UnsafeUnpin};
use std::marker::PhantomPinned;

fn is_unpin<T: Unpin>() {}

#[test]
fn unsafe_unpin() {
    #[pin_project(UnsafeUnpin)]
    pub struct Blah<T, U> {
        field1: U,
        #[pin]
        field2: Option<T>,
    }

    #[allow(unsafe_code)]
    unsafe impl<T: Unpin, U> UnsafeUnpin for Blah<T, U> {}

    is_unpin::<Blah<(), PhantomPinned>>();
}
