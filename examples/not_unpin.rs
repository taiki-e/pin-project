// See ./not_unpin-expanded.rs for generated code.

#![allow(dead_code)]
#![allow(clippy::extra_unused_type_parameters)] // https://github.com/rust-lang/rust-clippy/issues/10319

use pin_project::pin_project;

#[pin_project(!Unpin)]
pub struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

fn main() {
    fn _is_unpin<T: Unpin>() {}
    // _is_unpin::<Struct<(), ()>>(); //~ ERROR `std::marker::PhantomPinned` cannot be unpinned
}
