// run-pass

use pin_project::pin_project;

// FIXME
#[test]
fn unsafe_unpin() {
    #[pin_project(UnsafeUnpin)]
    pub struct Blah<T> {
        field_1: u8,
        #[pin]
        field_2: Option<T>,
    }
}

fn main() {}
