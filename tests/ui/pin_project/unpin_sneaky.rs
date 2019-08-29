use pin_project::pin_project;

#[pin_project]
struct Foo {
    #[pin]
    inner: u8
}

impl Unpin for __UnpinStructFoo {}

fn main() {}
