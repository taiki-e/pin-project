#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use core::pin::Pin;
use pin_project::{pin_project, pinned_drop};

#[test]
fn safe_project() {
    #[pin_project(PinnedDrop)]
    pub struct Foo<'a> {
        was_dropped: &'a mut bool,
        #[pin]
        field: u8,
    }

    #[pinned_drop]
    impl PinnedDrop for Foo<'_> {
        fn drop(mut self: Pin<&mut Self>) {
            **self.project().was_dropped = true;
        }
    }

    let mut was_dropped = false;
    drop(Foo { was_dropped: &mut was_dropped, field: 42 });
    assert!(was_dropped);
}
