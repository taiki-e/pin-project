use pin_project::pin_project;

#[pin_project(Replace)]
enum Enum<T, U> {
    V {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    None,
}

fn main() {}
