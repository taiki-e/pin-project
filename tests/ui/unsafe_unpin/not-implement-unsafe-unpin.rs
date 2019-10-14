use pin_project::pin_project;

#[pin_project(UnsafeUnpin)]
struct Foo<T, U> {
    #[pin]
    inner: T,
    other: U,
}

fn is_unpin<T: Unpin>() {}

fn main() {
    is_unpin::<Foo<(), ()>>(); //~ ERROR E0277
}
