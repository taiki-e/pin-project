use pin_project::pin_project;

#[pin_project]
enum Enum<T, U> {
    Pinned(#[pin] T),
    Unpinned(U),
}

fn main() {}
