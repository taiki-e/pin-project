// compile-fail

#![deny(warnings, unsafe_code)]

use pin_project::pin_project;

#[pin_project]
#[repr(packed, C)] //~ ERROR may not be used on #[repr(packed)] type
struct Foo {
    #[pin]
    field: u8,
}

// Test putting 'repr' before the 'pin_project' attribute
#[repr(packed, C)] //~ ERROR may not be used on #[repr(packed)] type
#[pin_project]
struct Foo2 {
    #[pin]
    field: u8,
}

fn main() {}
