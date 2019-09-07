// compile-fail

use pin_project::pin_project;

#[cfg(unix)]
pub struct Unix;
#[cfg(windows)]
pub struct Windows;

#[pin_project]
pub struct TupleStruct(
    #[cfg(unix)] //~ ERROR `cfg` attributes on the field of tuple structs are not supported
    #[pin]
    Unix,
    #[cfg(windows)]
    #[pin]
    Windows,
);

fn main() {}
