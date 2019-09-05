// compile-fail

use pin_project::pin_project;

#[pin_project]
struct A<T> {
    #[pin()] //~ ERROR unexpected token
    pinned: T,
}

#[pin_project]
struct B<T>(#[pin(foo)] T); //~ ERROR unexpected token

#[pin_project]
enum C<T> {
    A(#[pin(foo)] T), //~ ERROR unexpected token
}

#[pin_project]
enum D<T> {
    A {
        #[pin(foo)] //~ ERROR unexpected token
        pinned: T,
    },
}

#[pin_project(UnsafeUnpin,,)] //~ ERROR expected identifier
struct E<T> {
    #[pin]
    pinned: T,
}

#[pin_project(UnsafeUnpin, UnsafeUnpin)] //~ ERROR duplicate `UnsafeUnpin` argument
struct F<T> {
    #[pin]
    pinned: T,
}

#[pin_project(PinnedDrop, PinnedDrop)] //~ ERROR duplicate `PinnedDrop` argument
struct G<T> {
    #[pin]
    pinned: T,
}

#[pin_project(PinnedDrop, UnsafeUnpin, UnsafeUnpin)] //~ ERROR duplicate `UnsafeUnpin` argument
struct H<T> {
    #[pin]
    pinned: T,
}

#[pin_project(PinnedDrop, UnsafeUnpin, PinnedDrop, PinnedDrop)] //~ ERROR duplicate `PinnedDrop` argument
struct I<T> {
    #[pin]
    pinned: T,
}

fn main() {}
