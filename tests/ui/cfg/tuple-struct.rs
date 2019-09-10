// compile-fail

use pin_project::pin_project;

#[cfg(not(any()))]
pub struct Foo;
#[cfg(any())]
pub struct Bar;

#[pin_project]
pub struct TupleStruct(
    #[cfg(not(any()))] //~ ERROR `cfg` attributes on the field of tuple structs are not supported
    #[pin]
    Foo,
    #[cfg(any())]
    #[pin]
    Bar,
);

fn main() {}
