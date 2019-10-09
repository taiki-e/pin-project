use pin_project::pin_project;

#[pin_project(PinnedDrop)] //~ ERROR E0277
pub struct Foo {
    #[pin]
    field: u8,
}

fn main() {}
