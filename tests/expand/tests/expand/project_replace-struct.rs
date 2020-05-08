use pin_project::pin_project;

#[pin_project(Replace)]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

fn main() {}
