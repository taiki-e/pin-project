// compile-fail
#![allow(dead_code)]

use pin_project::pin_projectable;

#[pin_projectable]
#[repr(packed, C)] //~ ERROR may not be used on #[repr(packed)] type
struct Foo {
    #[pin]
    field: u8
}

#[pin_projectable]
#[repr(packed, C)] //~ ERROR may not be used on #[repr(packed)] type
enum Blah {
    Tuple(#[pin] u8)
}


fn main() {}
