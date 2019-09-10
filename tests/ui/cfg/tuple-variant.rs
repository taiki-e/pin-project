// compile-fail

use pin_project::pin_project;

#[cfg(not(any()))]
pub struct Foo;
#[cfg(any())]
pub struct Bar;

#[pin_project]
pub enum Field {
    Tuple(
        #[cfg(not(any()))] //~ ERROR `cfg` attributes on the field of tuple variants are not supported
        #[pin]
        Foo,
        #[cfg(any())]
        #[pin]
        Bar,
    ),
}

fn main() {}
