// compile-fail

#![deny(warnings)]


use pin_project::pin_projectable;

#[pin_projectable]
#[repr(packed, C)] //~ ERROR may not be used on #[repr(packed)] type
struct Foo {
    #[pin]
    field: u8,
}

// Test putting 'repr' before the 'pin_projectable' attribute
#[repr(packed, C)] //~ ERROR may not be used on #[repr(packed)] type
#[pin_projectable]
struct Foo2 {
    #[pin]
    field: u8,
}

fn main() {}
