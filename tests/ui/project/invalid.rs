// compile-fail

#![deny(warnings)]

use pin_project::{pin_projectable, project};

#[pin_projectable]
struct A<T> {
    #[pin]
    future: T,
}

#[project]
fn foo() {
    let x = A { future: 0 };
    #[project(foo)] //~ ERROR unexpected token
    let A { future } = x.project();
}

fn main() {}
