// compile-fail

use pin_project::pin_project;

#[cfg(unix)]
pub struct Unix;
#[cfg(windows)]
pub struct Windows;

#[pin_project]
pub enum Field {
    Tuple(
        #[cfg(unix)] //~ ERROR `cfg` attributes on the field of tuple variants are not supported
        #[pin]
        Unix,
        #[cfg(windows)]
        #[pin]
        Windows,
    ),
}

fn main() {}
