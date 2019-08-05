// compile-fail

#![deny(warnings)]

use pin_project::pin_projectable;

#[pin_projectable]
struct A<T> {
    #[pin()] //~ ERROR unexpected token
    future: T,
}

#[pin_projectable]
struct B<T>(#[pin(foo)] T); //~ ERROR unexpected token

#[pin_projectable]
enum C<T> {
    A(#[pin(foo)] T), //~ ERROR unexpected token
}

#[pin_projectable]
enum D<T> {
    A {
        #[pin(foo)] //~ ERROR unexpected token
        foo: T,
    },
}

fn main() {}
