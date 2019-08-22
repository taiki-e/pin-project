// compile-fail

#![deny(warnings, unsafe_code)]

use pin_project::pin_project;

#[pin_project]
struct A<T> {
    #[pin()] //~ ERROR unexpected token
    future: T,
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
        foo: T,
    },
}

#[pin_project(UnsafeUnpin,,)] //~ ERROR unexpected token
struct E<T> {
    #[pin]
    future: T,
}

fn main() {}
