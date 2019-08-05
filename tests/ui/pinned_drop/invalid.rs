// compile-fail

#![deny(warnings)]

use pin_project::pin_project;
use std::pin::Pin;

pin_project! {
    #[pin_projectable]
    pub struct Foo<'a> {
        was_dropped: &'a mut bool,
        #[pin] field_2: u8
    }

    #[pinned_drop(foo)] //~ ERROR unexpected token
    fn do_drop(_foo: Pin<&mut Foo<'_>>) {}
}

fn main() {}
