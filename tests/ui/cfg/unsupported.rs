// compile-fail

use pin_project::pin_project;

#[pin_project]
struct Struct1 {
    //~^ ERROR may not be used on structs with zero fields
    #[cfg(any())]
    #[pin]
    f: u8,
}

fn main() {}
