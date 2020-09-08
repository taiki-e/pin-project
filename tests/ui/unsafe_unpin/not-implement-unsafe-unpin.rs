use pin_project::pin_project;

#[pin_project(UnsafeUnpin)]
struct Struct<T, U> {
    #[pin]
    f1: T,
    f2: U,
}

fn is_unpin<T: Unpin>() {}

fn main() {
    is_unpin::<Struct<(), ()>>(); //~ ERROR E0277
}
