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

#[pin_project]
enum F<T> {
    #[pin] //~ ERROR may only be used on fields of structs or variants
    A(T),
}

#[pin_project]
#[pin] //~ ERROR may only be used on fields of structs or variants
enum G<T> {
    A(T),
}

fn main() {}
