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

    #[pinned_drop]
    fn do_drop(foo: Pin<&mut Foo<'_>>) -> bool { //~ ERROR #[pinned_drop] function must return the unit type
        **foo.project().was_dropped = true;
        true
    }
}

pin_project! {
    #[pin_projectable]
    pub struct Bar<'a> {
        was_dropped: &'a mut bool,
        #[pin] field_2: u8
    }

    #[pinned_drop]
    fn do_drop(foo: Pin<&mut Bar<'_>>) -> () { // OK
        **foo.project().was_dropped = true;
    }
}

fn main() {}
